use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SipmPlane {
    pub sipm_size   : f64,
    pub sipm_area   : f64,
    pub sipm_gap    : f64,
    pub n_sipms_side: usize,
}


impl SipmPlane {
    pub fn sipm_pitch(&self) -> f64 {
        self.sipm_size + self.sipm_gap
    }

    pub fn sipm_pos(&self) -> Vec<f64> {
        let n = self.n_sipms_side/2;
        let mut v : Vec<f64> =
            (0..n).into_iter()
                  .map      (|i| (i as f64  + 0.5) * self.sipm_pitch())
                  .flat_map (|p| [-p, p].into_iter())
                  .collect  ();
        v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        v
    }

    pub fn sipm_bins(&self) -> Vec<f64> {
        let v = self.sipm_pos();
        let inner_bins = [ -self.sipm_size/2. - self.sipm_gap/2.
                         , -self.sipm_size/2.
                         ,  self.sipm_size/2.];
        let mut bins : Vec<f64> =
            v.iter    ()
             .flat_map(|p| inner_bins.into_iter().map(move |ip| p + ip))
             .collect ();
        bins.push(bins.last().unwrap() + self.sipm_size/2. + self.sipm_gap/2.);
        bins
    }
}


#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use super::*;

    fn test_plane() -> SipmPlane {
        SipmPlane{ sipm_size: 6.0
                 , sipm_area: 5.85 * 5.95
                 , sipm_gap: 0.5
                 , n_sipms_side: 10
                 }
    }

    #[test]
    fn sipm_pos() {
        let plane = test_plane();
        let pos = &plane.sipm_pos();

        assert_eq!(pos.len(), 10);
        assert_float_eq!(-pos.first().unwrap(), pos.last().unwrap()   , ulps<=2);
        assert_float_eq!(      pos[2] - pos[1],  plane.sipm_pitch()   , ulps<=2);
        assert_float_eq!(               pos[5],  plane.sipm_pitch()/2., ulps<=2);

    }
}
