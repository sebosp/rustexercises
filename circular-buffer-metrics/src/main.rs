//! Loads prometheus metrics every now and then and displays stats
use futures::future::lazy;
use futures::sync::mpsc;
use log::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;
use tokio_core::reactor::Core;

fn main() {
    println!("Starting program");
    env_logger::init();
    info!("Loading async core");
    tokio::run(lazy(|| {
        info!("Loading async core");
        let mut core = Core::new().unwrap();
        let core_handle = &core.handle();
        let config_location = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/charts.yml"));
        let mut config_res = circular_buffer_metrics::config::Config::read_config(&config_location);
        let mut config = match config_res {
            Err(err) => {
                error!(
                    "Unable to load config from file: {:?}: '{}'",
                    config_location, err
                );
                return Ok(());
            }
            Ok(ref mut config) => {
                info!("Loaded config from file: {:?}", config_location);
                for chart in &mut config.charts {
                    debug!("Loading chart config with name: '{}'", chart.name);
                    for series in &mut chart.sources {
                        debug!(" - Loading series with name: '{}'", chart.name);
                        if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(
                            ref mut prom,
                        ) = series
                        {
                            match prom.set_url() {
                                Ok(()) => {
                                    debug!(" - Parsed URL '{}'", prom.source);
                                    prom.tokio_core = Some(&core_handle);
                                }
                                Err(err) => error!(" - Parsing URL '{}': '{}'", err, prom.source),
                            };
                        } else {
                            debug!(" - Not a Prometheus Time Series, will not load tokio core");
                        }
                    }
                }
                config
            }
        };
        // Create the channel that is used to communicate with the
        // background task.
        for chart in &mut config.charts {
            debug!("Loading chart series with name: '{}'", chart.name);
            for series in &mut chart.sources {
                if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(
                    ref mut prom,
                ) = series
                {
                    debug!(" - Found time_series, adding interval run");
                    let url = prom.url.clone();
                    let interval = Interval::new(Instant::now(), Duration::from_secs(10))
                        .take(10) //  Test 10 times first
                        .map_err(|e| panic!("interval errored; err={:?}", e))
                        .fold(url, move |url, instant| {
                            println!("fire; instant={:?}", instant);
                            let get_prom_data_task =
                                circular_buffer_metrics::prometheus::get_from_prometheus(
                                    url.clone(),
                                )
                                .and_then(|value| {
                                    let res =
                                        circular_buffer_metrics::prometheus::parse_json(&value);
                                    debug!("res={:?}", res);
                                    Ok(())
                                });
                            tokio::spawn(get_prom_data_task);
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
                            Ok(url)
                        });
                    tokio::spawn(interval);
                }
            }
        }
        Ok(())
    }));
    info!("Exiting.");
}
