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
    let mut config_res = circular_buffer_metrics::config::Config::read_config(&config_location);
    let mut config = match config_res {
        Err(err) => {
            error!(
                "Unable to load config from file: {:?}: {}",
                config_location, err
            );
            return;
        }
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
            config
        }
    };
    for chart in &mut config.charts {
        debug!("Loading chart series {}", chart.name);
        for series in &mut chart.sources {
            if let circular_buffer_metrics::TimeSeriesSource::PrometheusTimeSeries(ref mut prom) =
                series
            {
                let chart_data = core.run(prom.get_from_prometheus());
                if let Ok(Some(prom_response)) = chart_data {
                    match prom.load_prometheus_response(prom_response) {
                        Ok(num_records) => {
                            info!("Loaded {} records", num_records);
                        }
                        Err(err) => {
                            error!("Error loading prometheus response: {}", err);
                        }
                    }
                }
            }
        }
    }
    info!("Exiting.");
}
