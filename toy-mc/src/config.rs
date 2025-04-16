use serde::{Serialize, Deserialize};
use config::{Config, ConfigError, File};

use crate::{Geometry, SimParams};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimConfig {
    pub geometry  : Geometry,
    pub sim_params: SimParams,
    pub n_events  : usize,
    pub output    : String,
}

impl SimConfig {
    pub fn new(filename: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(filename))
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }

    pub fn override_n_events(self, n_events: usize) -> Self {
        Self{n_events, ..self}
    }

    pub fn override_output(self, output: String) -> Self {
        Self{output, ..self}
    }

    pub fn overrides(self, n_events: Option<usize>, output: Option<String>) -> Self {
        let conf = self;
        let conf = match n_events {
            Some(n) => conf.override_n_events(n),
            None    => conf,
        };
        let conf = match output {
            Some(p) => conf.override_output(p),
            None    => conf,
        };
        conf
    }
}
