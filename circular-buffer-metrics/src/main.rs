//! Loads prometheus metrics every now and then and displays stats
use env_logger::Env;
use futures::future::lazy;
use futures::sync::mpsc;
use log::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;

fn load_config_file() -> circular_buffer_metrics::config::Config {
    let config_location = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/charts.yml"));
    let config_res = circular_buffer_metrics::config::Config::read_config(&config_location);
    match config_res {
        Err(err) => {
            error!(
                "Unable to load config from file: {:?}: '{}'",
                config_location, err
            );
            circular_buffer_metrics::config::Config::default()
        }
        Ok(config) => {
            info!("Loaded config from file: {:?}", config_location);
            for chart in &config.charts {
                debug!("Loading chart config with name: '{}'", chart.name);
                for series in &chart.sources {
                    debug!(" - Loading series with name: '{}'", series.name());
                }
            }
            config
        }
    }
}

/// `AsyncMetricItemData` contains a way to address a response to a particular
/// item in our TimeSeriesCharts
#[derive(Debug, Clone)]
pub struct AsyncMetricItemData {
    pull_interval: u64,
    source_url: String,
    chart_index: usize,  // For Vec<TimeSeriesChart>
    series_index: usize, // For Vec<TimeSeriesSource>
    data: Option<circular_buffer_metrics::prometheus::HTTPResponse>,
    capacity: usize,
}

fn async_coordinator(
    rx: mpsc::Receiver<AsyncMetricItemData>,
    charts: Vec<circular_buffer_metrics::TimeSeriesChart>,
) -> impl Future<Item = (), Error = ()> {
    rx.for_each(move |response| {
        debug!("Coordinator Got response: {:?}", response);
        Ok(())
    })
}

/// `fetch_prometheus_response` creates intervals for each series requested
/// Each series will have to reply to a mspc tx with the data
fn fetch_prometheus_response(
    item: AsyncMetricItemData,
    tx: mpsc::Sender<AsyncMetricItemData>,
) -> impl Future<Item = (), Error = ()> {
    let url = circular_buffer_metrics::prometheus::PrometheusTimeSeries::prepare_url(
        &item.source_url,
        item.capacity as u64,
    )
    .unwrap();
    circular_buffer_metrics::prometheus::get_from_prometheus(url.clone())
        .timeout(Duration::from_secs(item.pull_interval))
        .map_err(|e| error!("get_from_prometheus; err={:?}", e))
        .and_then(move |value| {
            debug!("Got prometheus raw value={:?}", value);
            let res = circular_buffer_metrics::prometheus::parse_json(&value);
            debug!("Parsed JSON to res={:?}", res);
            tx.send(AsyncMetricItemData {
                source_url: item.source_url.clone(),
                chart_index: item.chart_index,
                series_index: item.series_index,
                pull_interval: item.pull_interval,
                data: res.clone(),
                capacity: item.capacity,
            })
            .map_err(|e| {
                error!(
                    "fetch_prometheus_response: send data back to coordinator; err={:?}",
                    e
                )
            })
            .and_then(|res| {
                debug!("Got res={:?}", res);
                Ok(())
            })
        })
        .map_err(|e| error!("Sending result to coordinator; err={:?}", e))
}
/// `spawn_interval_polls` creates intervals for each series requested
/// Each series will have to reply to a mspc tx with the data
fn spawn_interval_polls(
    item: &AsyncMetricItemData,
    tx: mpsc::Sender<AsyncMetricItemData>,
) -> impl Future<Item = (), Error = ()> {
    Interval::new(Instant::now(), Duration::from_secs(item.pull_interval))
        .take(10) //  Test 10 times first
        .map_err(|e| panic!("interval errored; err={:?}", e))
        .fold(
            AsyncMetricItemData {
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
                // .map_err(|e| panic!("Get from prometheus err={:?}", e));
                //                            .and_then(|value| {
                //                                if let Some(prom_response) = value {
                //                                    match prom.load_prometheus_response(prom_response) {
                //                                        Ok(num_records) => {
                //                                            info!(" - Loaded {} records", num_records);
                //                                        }
                //                                        Err(err) => {
                //                                            error!(
                //                                                " - Error loading prometheus response: '{}'",
                //                                                err
                //                                            );
                //                                        }
                //                                    }
                //                                }
                //                            });
                //Ok(prom)
            },
        )
        .map(|_| ())
}
fn main() {
    println!("Starting program");
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let config = load_config_file();
    let mut chart_index = 0usize;
    tokio::run(lazy(move || {
        // Create the channel that is used to communicate with the
        // background task.
        let (tx, rx) = mpsc::channel(4_096usize);
        let charts = config.charts.clone();
        tokio::spawn(lazy(move || async_coordinator(rx, charts)));
        for chart in config.charts {
            debug!("Loading chart series with name: '{}'", chart.name);
            let mut series_index = 0usize;
            for series in chart.sources {
                if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(ref prom) =
                    series
                {
                    debug!(" - Found time_series, adding interval run");
                    let data_request = AsyncMetricItemData {
                        source_url: prom.source.clone(),
                        pull_interval: prom.pull_interval as u64,
                        chart_index,
                        series_index,
                        capacity: prom.series.metrics_capacity,
                        data: None,
                    };
                    let tx = tx.clone();
                    tokio::spawn(lazy(move || spawn_interval_polls(&data_request, tx)));
                }
                series_index += 1;
            }
            chart_index += 1;
        }
        Ok(())
    }));
    println!("Exiting.");
}
