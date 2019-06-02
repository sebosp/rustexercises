//! Reading configuration from a yaml file
use serde::{self, de, Deserialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
static DEFAULT_CHART_CONFIG: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/charts.yml"));
/// Top-level config type
#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    /// Initial dimensions
    #[serde(default, deserialize_with = "failure_default")]
    charts: Option<Vec<crate::TimeSeriesChart>>,
}
fn failure_default<'a, D, T>(deserializer: D) -> ::std::result::Result<T, D::Error>
where
    D: de::Deserializer<'a>,
    T: Deserialize<'a> + Default,
{
    match T::deserialize(deserializer) {
        Ok(value) => Ok(value),
        Err(err) => {
            error!("Problem with config: {}; using default value", err);
            Ok(T::default())
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        serde_yaml::from_str(DEFAULT_CHART_CONFIG).expect("default config is invalid")
    }
}
impl Config {
    fn read_config(path: &PathBuf) -> Result<Config, String> {
        let mut contents = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        // Prevent parsing error with empty string
        if contents.is_empty() {
            return Ok(Config::default());
        }

        let config: Config = serde_yaml::from_str(&contents).unwrap();

        Ok(config)
    }
}
