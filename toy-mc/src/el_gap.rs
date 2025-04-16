use serde::{Deserialize, Serialize};
use derive_new::new;

#[derive(new, Debug, Deserialize, Serialize, Clone)]
pub struct ElGap {
    pub el_r        : f64,
    pub el_gap_front: f64,
    pub el_gap_back : f64,
}

#[cfg(test)]
mod tests {
    // use super::*;
}
