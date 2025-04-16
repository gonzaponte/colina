use serde::{Deserialize, Serialize};
use derive_new::new;

use crate::sipm_plane::SipmPlane;
use crate::wire_plane::WirePlane;
use crate::el_gap    ::ElGap;

#[derive(new, Debug, Deserialize, Serialize, Clone)]
pub struct Geometry {
    pub wire_plane: WirePlane,
    pub sipm_plane: SipmPlane,
    pub el_gap    : ElGap,
    pub buffer    : f64
}

#[cfg(test)]
mod tests {
    // use super::*;
}
