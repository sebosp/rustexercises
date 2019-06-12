//! Reading configuration from a yaml file
use log::*;
use serde_yaml;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
static DEFAULT_CHART_CONFIG: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/charts.yml"));
/// Top-level config type
#[derive(Debug, PartialEq, Deserialize)]
pub struct Config<'a> {
    pub charts: Vec<crate::TimeSeriesChart<'a>>,
}
impl<'a> Default for Config<'a> {
    fn default() -> Self {
        serde_yaml::from_str(DEFAULT_CHART_CONFIG).expect("default config is invalid")
    }
}
impl<'a> Config<'a> {
    /// This method is used from config/mod.rs in Alacritty.
    /// This is a copy for testing
    pub fn read_config(path: &PathBuf) -> Result<Config, String> {
        let mut contents = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        // Prevent parsing error with empty string
        if contents.is_empty() {
            info!("Config file is empty, using defaults");
            return Ok(Config::default());
        }

        let config: Config = serde_yaml::from_str(&contents).unwrap();

        Ok(config)
    }
}
