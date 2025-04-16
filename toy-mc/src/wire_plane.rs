use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WirePlane {
    pub wire_pitch   : f64,
    pub wire_r       : f64,
    pub wire_rotation: f64,
    pub n_wires      : usize,
}

impl WirePlane {
    pub fn wire_pos(&self) -> Vec<f64> {
        let n = self.n_wires/2;
        let mut v : Vec<f64> =
            (0..n).into_iter()
                  .map      (|i| (i as f64 + 0.5) * self.wire_pitch)
                  .flat_map (|p| [-p, p].into_iter())
                  .collect  ();
        v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        v
    }
}


#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use super::*;

    fn test_plane() -> WirePlane {
        WirePlane{
            wire_pitch   : 5.0,
            wire_r       : 5.0e-3,
            wire_rotation: 45.0,
            n_wires      : 14,
        }
    }

    #[test]
    fn wire_pos() {
        let plane = test_plane();
        let pos   = plane.wire_pos();

        assert_eq!(pos.len(), 14);
        assert_float_eq!(-pos.first().unwrap(), pos.last().unwrap(), ulps<=2);
        assert_float_eq!(      pos[2] - pos[1],    plane.wire_pitch, ulps<=2);
        assert_float_eq!(               pos[7], plane.wire_pitch/2., ulps<=2);
    }

}
