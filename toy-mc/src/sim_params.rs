use serde::{Deserialize, Serialize};
use derive_new::new;

#[derive(new, Debug, Deserialize, Serialize, Clone)]
pub struct SimParams {
    pub dep_energy : f64,
    pub w_i        : f64,
    pub light_yield: f64,
    pub el_range   : f64,
    pub cloud_r    : f64,
    pub fano_factor: f64,
}

impl SimParams {
    pub fn n_ie_ave(&self) -> f64 {
        self.dep_energy / self.w_i
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
