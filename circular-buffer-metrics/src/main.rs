//! Loads prometheus metrics every now and then and displays stats
use circular_buffer_metrics::config::Config;
use circular_buffer_metrics::prometheus;
use circular_buffer_metrics::SizeInfo; // XXX: remove on merge.
use circular_buffer_metrics::TimeSeriesChart;
use circular_buffer_metrics::TimeSeriesSource;
use env_logger::Env;
use futures::future::lazy;
use futures::sync::{mpsc, oneshot};
use log::*;
use std::thread;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;

/// `MetricRequest` contains a way to address a particular
/// item in our TimeSeriesCharts vectors
#[derive(Debug, Clone)]
pub struct MetricRequest {
    pull_interval: u64,
    source_url: String,
    chart_index: usize,  // For Vec<TimeSeriesChart>
    series_index: usize, // For Vec<TimeSeriesSource>
    data: Option<prometheus::HTTPResponse>,
    capacity: usize, // This maps to the time range in seconds to query.
}

/// `AsyncChartTask` contains message types that async_coordinator can work on
#[derive(Debug)]
pub enum AsyncChartTask {
    LoadResponse(MetricRequest),
    GetOpenGL(usize, oneshot::Sender<Vec<f32>>),
}

/// `load_http_response` is called by async_coordinator when a task of type
/// LoadResponse is received
pub fn load_http_response(charts: &mut Vec<TimeSeriesChart>, response: MetricRequest) {
    if let Some(data) = response.data {
        if response.chart_index < charts.len()
            && response.series_index < charts[response.chart_index].sources.len()
        {
            if let TimeSeriesSource::PrometheusTimeSeries(ref mut prom) =
                charts[response.chart_index].sources[response.series_index]
            {
                match prom.load_prometheus_response(data) {
                    Ok(num_records) => {
                        info!(
                            "Loaded {} records from {} into TimeSeries",
                            num_records, response.source_url
                        );
                    }
                    Err(err) => {
                        debug!(
                            "Error from {} into TimeSeries: {:?}",
                            response.source_url, err
                        );
                    }
                }
            }
            charts[response.chart_index].update_opengl_vecs(SizeInfo {
                padding_x: 0.,
                padding_y: 0.,
                height: 100.,
                width: 100.,
                ..SizeInfo::default()
            });
        }
    }
}

/// `get_opengl_vecs` is called by async_coordinator when an task or type GetOpenGL
/// is received, it should contain the chart index to represent
pub fn get_opengl_vecs(
    charts: &[TimeSeriesChart],
    chart_index: usize,
    channel: oneshot::Sender<Vec<f32>>,
) {
    debug!("get_opengl_vecs for chart_index: {}", chart_index);
    match channel.send(if chart_index > charts.len() {
        vec![]
    } else {
        charts[chart_index].series_opengl_vecs.clone()
    }) {
        Ok(()) => {
            if chart_index > charts.len() {
                debug!(
                    "get_opengl_vecs: oneshot::message sent for {}[OutOfBounds]",
                    chart_index
                );
            } else {
                debug!(
                    "get_opengl_vecs: oneshot::message sent for {}[InsideBounds]",
                    chart_index
                );
            }
        }
        Err(err) => error!("get_opengl_vecs: Error sending: {:?}", err),
    };
    /*.and_then(|_| {
        let vec_len = charts[chart_index].series_opengl_vecs.len();
        debug!("Sent vec with {} items", vec_len);
        Ok(vec_len)
    })
    .map_err(|e| {
        error!("get_opengl_vecs; err={:?}", e);
    });*/
}

/// `async_coordinator` receives messages from the tasks about data loaded from
/// the network, it owns the charts data.
fn async_coordinator(
    rx: mpsc::Receiver<AsyncChartTask>,
    mut charts: Vec<TimeSeriesChart>,
) -> impl Future<Item = (), Error = ()> {
    debug!("async_coordinator: Starting");
    rx.for_each(move |message| {
        info!("Coordinator Got message: {:?}", message);
        match message {
            AsyncChartTask::LoadResponse(req) => load_http_response(&mut charts, req),
            AsyncChartTask::GetOpenGL(chart_index, channel) => {
                get_opengl_vecs(&charts, chart_index, channel);
            }
        };
        Ok(())
    })
}

/// `fetch_prometheus_response` gets data from prometheus and once data is ready
/// it sends the results to the coordinator.
fn fetch_prometheus_response(
    item: MetricRequest,
    tx: mpsc::Sender<AsyncChartTask>,
) -> impl Future<Item = (), Error = ()> {
    debug!("fetch_prometheus_response: Starting");
    let url = prometheus::PrometheusTimeSeries::prepare_url(&item.source_url, item.capacity as u64)
        .unwrap();
    prometheus::get_from_prometheus(url.clone())
        .timeout(Duration::from_secs(item.pull_interval))
        .map_err(|e| error!("get_from_prometheus; err={:?}", e))
        .and_then(move |value| {
            debug!("Got prometheus raw value={:?}", value);
            let res = prometheus::parse_json(&value);
            debug!("Parsed JSON to res={:?}", res);
            tx.send(AsyncChartTask::LoadResponse(MetricRequest {
                source_url: item.source_url.clone(),
                chart_index: item.chart_index,
                series_index: item.series_index,
                pull_interval: item.pull_interval,
                data: res.clone(),
                capacity: item.capacity,
            }))
            .map_err(|e| {
                error!(
                    "fetch_prometheus_response: send data back to coordinator; err={:?}",
                    e
                )
            })
            .and_then(|res| {
                debug!("fetch_prometheus_response: res={:?}", res);
                Ok(())
            })
        })
        .map_err(|e| error!("Sending result to coordinator; err={:?}", e))
}
/// `spawn_interval_polls` creates intervals for each series requested
/// Each series will have to reply to a mspc tx with the data
fn spawn_interval_polls(
    item: &MetricRequest,
    tx: mpsc::Sender<AsyncChartTask>,
) -> impl Future<Item = (), Error = ()> {
    debug!("spawn_interval_polls: Starting for item={:?}", item);
    Interval::new(Instant::now(), Duration::from_secs(item.pull_interval))
        //.take(10) //  Test 10 times first
        .map_err(|e| panic!("interval errored; err={:?}", e))
        .fold(
            MetricRequest {
                source_url: item.source_url.clone(),
                chart_index: item.chart_index,
                series_index: item.series_index,
                pull_interval: item.pull_interval,
                data: None,
                capacity: item.capacity,
            },
            move |async_metric_item, instant| {
                debug!(
                    "Interval triggered for {:?} at instant={:?}",
                    async_metric_item.source_url, instant
                );
                fetch_prometheus_response(async_metric_item.clone(), tx.clone()).and_then(|res| {
                    debug!("Got response {:?}", res);
                    Ok(async_metric_item)
                })
            },
        )
        .map(|_| ())
}
fn main() {
    println!("Starting program");
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let config = Config::load_config_file();
    let charts = config.charts.clone();
    let collected_data = config.clone();
    let mut chart_index = 0usize;
    // Create the channel that is used to communicate with the
    // background task.
    let (tx, rx) = mpsc::channel(4_096usize);
    let poll_tx = tx.clone();
    tokio::run(lazy(move || {
        tokio::spawn(lazy(move || async_coordinator(rx, charts)));
        for chart in config.charts {
            debug!("Loading chart series with name: '{}'", chart.name);
            let mut series_index = 0usize;
            for series in chart.sources {
                if let TimeSeriesSource::PrometheusTimeSeries(ref prom) = series {
                    debug!(" - Found time_series, adding interval run");
                    let data_request = MetricRequest {
                        source_url: prom.source.clone(),
                        pull_interval: prom.pull_interval as u64,
                        chart_index,
                        series_index,
                        capacity: prom.series.metrics_capacity,
                        data: None,
                    };
                    let poll_tx = poll_tx.clone();
                    tokio::spawn(lazy(move || spawn_interval_polls(&data_request, poll_tx)));
                }
                series_index += 1;
            }
            chart_index += 1;
        }
        let mut counter = 0;
        loop {
            let one_second = Duration::from_secs(1);
            info!("Starting to load data from async_coordinator");
            thread::sleep(one_second);
            for idx in 0..collected_data.charts.len() {
                let (opengl_tx, opengl_rx) = oneshot::channel();
                let get_opengl_task = tx
                    .clone()
                    .send(AsyncChartTask::GetOpenGL(idx, opengl_tx))
                    .map_err(|e| error!("Sending GetOpenGL Task: err={:?}", e))
                    .and_then(move |_res| {
                        debug!("Sent GetOpenGL Task for chart index: {}", idx);
                        Ok(())
                    });
                tokio::spawn(lazy(|| get_opengl_task));
                let opengl_rx = opengl_rx.map(|x| x);
                match opengl_rx.wait() {
                    Ok(data) => {
                        debug!("Got response from GetOpenGL Task: {:?}", data);
                    }
                    Err(err) => {
                        error!("Error response from GetOpenGL Task: {:?}", err);
                    }
                }
            }
            if counter > 100 {
                break;
            }
            counter += 1;
        }
        Ok(())
    }));
    println!("Exiting.");
}
