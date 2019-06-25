//! Loads prometheus metrics every now and then and displays stats
use env_logger::Env;
use futures::future::lazy;
use futures::sync::{mpsc, oneshot};
use log::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;

/// `AsyncMetricItemLocator` contains a way to address a response to a particular
/// item in our TimeSeriesCharts
#[derive(Debug)]
pub struct AsyncMetricItemLocator {
    pull_interval: u64,
    url: hyper::Uri,
    chart_index: usize,  // For Vec<TimeSeriesChart>
    series_index: usize, // For Vec<TimeSeriesSource>
}

type ResponseMessage = oneshot::Sender<hyper::Chunk>;

fn async_coordinator(rx: mpsc::Receiver<ResponseMessage>) -> impl Future<Item = (), Error = ()> {
    rx.for_each(move |response| {
        info!("Coordinator Got response: {:?}", response);
        Ok(())
    })
}

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

/// `fetch_prometheus_response` creates intervals for each series requested
/// Each series will have to reply to a mspc tx with the data
fn fetch_prometheus_response(
    pull_interval: u64,
    url: hyper::Uri,
    tx: mpsc::Sender<ResponseMessage>,
) -> impl Future<Item = (), Error = ()> {
    circular_buffer_metrics::prometheus::get_from_prometheus(url.clone())
        .timeout(Duration::from_secs(pull_interval))
        .map_err(|e| panic!("get_from_prometheus; err={:?}", e))
        .and_then(|value| {
            let res = circular_buffer_metrics::prometheus::parse_json(&value);
            debug!("res={:?}", res);
            Ok(())
        })
}
/// `spawn_interval_polls` creates intervals for each series requested
/// Each series will have to reply to a mspc tx with the data
fn spawn_interval_polls(
    item: AsyncMetricItemLocator,
    tx: &mpsc::Sender<ResponseMessage>,
) -> impl Future<Item = (), Error = ()> {
    Interval::new(Instant::now(), Duration::from_secs(item.pull_interval))
        .take(10) //  Test 10 times first
        .map_err(|e| panic!("interval errored; err={:?}", e))
        .fold(
            (item.pull_interval, item.url.clone()),
            move |(pull_interval, url), instant| {
                debug!("Interval triggered for {:?} at instant={:?}", url, instant);
                tokio::spawn(fetch_prometheus_response(pull_interval, url, &tx));
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
                Ok((pull_interval, url))
            },
        )
        .map(|_| ())
}
fn main() {
    println!("Starting program");
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    tokio::run(lazy(|| {
        // Create the channel that is used to communicate with the
        // background task.
        let (tx, rx) = mpsc::channel(4_096usize);
        tokio::spawn(async_coordinator(rx));
        let mut config = load_config_file();
        let mut chart_index = 0usize;
        for chart in &mut config.charts {
            debug!("Loading chart series with name: '{}'", chart.name);
            let mut series_index = 0usize;
            for series in &mut chart.sources {
                if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(
                    ref mut prom,
                ) = series
                {
                    debug!(" - Found time_series, adding interval run");
                    match prom.set_url() {
                        Ok(()) => {
                            debug!(" - Parsed URL '{}'", prom.source);
                        }
                        Err(err) => error!(" - Parsing URL '{}': '{}'", err, prom.source),
                    };
                    let data_request = AsyncMetricItemLocator {
                        url: prom.url.clone(),
                        pull_interval: prom.pull_interval as u64,
                        chart_index,
                        series_index,
                    };
                    tokio::spawn(spawn_interval_polls(data_request, &tx));
                }
                series_index += 1;
            }
            chart_index += 1;
        }
        Ok(())
    }));
    println!("Exiting.");
}
