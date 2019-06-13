//! Loads prometheus metrics every now and then and displays stats
use log::*;
use std::path::PathBuf;
use tokio_core::reactor::Core;

fn main() {
    println!("Starting program");
    info!("Loading async core");
    let mut core = Core::new().unwrap();
    let core_handle = &core.handle();
    let config_location = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/charts.yml"));
    match circular_buffer_metrics::config::Config::read_config(&config_location) {
        Ok(ref mut config) => {
            info!("Loaded config from file: {:?}", config_location);
            for chart in &mut config.charts {
                debug!("Loading chart config {}", chart.name);
                for series in &mut chart.sources {
                    if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(
                        ref mut prom,
                    ) = series
                    {
                        debug!("Adding core for {}", prom.name);
                        prom.tokio_core = Some(&core_handle);
                    }
                }
            }
        }
        Err(err) => {
            error!(
                "Unable to load config from file: {:?}: {}",
                config_location, err
            );
        }
    };
    info!("Exiting.");
}
