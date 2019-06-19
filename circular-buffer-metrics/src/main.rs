//! Loads prometheus metrics every now and then and displays stats
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
            return;
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
    for chart in &mut config.charts {
        debug!("Loading chart series with name: '{}'", chart.name);
        for series in &mut chart.sources {
            if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(ref mut prom) =
                series
            {
                debug!(" - Found time_series, adding interval run");
                let interval = Interval::new(
                    Instant::now(),
                    Duration::from_secs(prom.pull_interval as u64),
                )
                .take(10) // Test 10 times first
                .map_err(|e| panic!("interval errored; err={:?}", e))
                .fold(prom, move |mut prom, instant| {
                    println!("fire; instant={:?}", instant);
                    prom.get_from_prometheus().and_then(move |value| {
                        if let Some(prom_response) = value {
                            match prom.load_prometheus_response(prom_response) {
                                Ok(num_records) => {
                                    info!(" - Loaded {} records", num_records);
                                }
                                Err(err) => {
                                    error!(" - Error loading prometheus response: '{}'", err);
                                }
                            }
                        }
                    });
                    Ok(prom)
                });
            }
        }
    }
    info!("Exiting.");
}
