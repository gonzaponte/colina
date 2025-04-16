use nalgebra::{Point2, DMatrix};
use ndhistogram::{Histogram, Hist2D, axis::VariableNoFlow, ndhistogram};

pub struct Image {
    n_bins: usize,
    hist  : Hist2D<VariableNoFlow<f64>, VariableNoFlow<f64>, usize>,
}

impl Image {
    pub fn new(bins: &Vec<f64>) -> Self {
        let n_bins = bins.len() - 1;
        let vbins  = VariableNoFlow::new(bins.clone()).unwrap();
        let hist   = ndhistogram!(vbins.clone(), vbins; usize);

        Self{ n_bins, hist }
    }

    pub fn fill(&mut self, p: &Point2<f64>) {
        self.hist.fill(&(p.x, p.y));
    }

    pub fn data(&self) -> Vec<usize> {
        self.hist.iter().map(|b| b.value).cloned().collect()
    }

    pub fn finalize(&self) -> DMatrix<usize> {
        // Read column-wise because DMatrix
        let n = self.n_bins;
        let v : Vec<usize> =
            self.hist
                .iter()
                .enumerate()
                .map   (|(i,v  )| (i/n, i.rem_euclid(n), v))
                .filter(|(r,_,_)| (*r > 0) & (*r < n-1)) // skip first and last rows
                .filter(|(_,c,_)| (*c > 0) & (*c < n-1)) // skip first and last columns
                .filter(|(r,_,_)| (r-1).rem_euclid(3) == 0)
                .filter(|(_,c,_)| (c-1).rem_euclid(3) == 0)
                .map   (|(_,_,b)| b.value)
                .copied()
                .collect();
        let n = n/3;
        DMatrix::from_vec(n, n, v).transpose()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use nalgebra::point;

    #[test]
    fn simple() {
        let bins  = vec![-7.0, -6.5, -0.5, 0.0, 0.5, 6.5, 7.0];
        let mut hist = Image::new(&bins);
        hist.fill(&point!(-6.0, -5.5)); // bin 0
        hist.fill(&point!( 5.5, -5.0)); // bin 1
        hist.fill(&point!( 4.0, -4.5));
        hist.fill(&point!(-3.5,  3.5)); // bin 2
        hist.fill(&point!(-3.0,  2.5));
        hist.fill(&point!(-2.5,  2.0));
        hist.fill(&point!( 6.0,  1.0)); // bin 3
        hist.fill(&point!( 1.0,  1.5));
        hist.fill(&point!( 0.5,  0.5));
        hist.fill(&point!( 2.0,  1.5));
        let m = hist.finalize();

        assert_eq!(m.shape(), (2,2) );
        assert_eq!(m[(0, 0)], 1);
        assert_eq!(m[(0, 1)], 2);
        assert_eq!(m[(1, 0)], 3);
        assert_eq!(m[(1, 1)], 4);
    }

    #[test]
    fn hist_order() {
        let bins  = vec![-0.5, 0.0, 0.5];
        let mut hist = Image::new(&bins);
        for _ in 0..1 { hist.fill(&point!(-0.1, -0.1)); } // bin 0
        for _ in 0..2 { hist.fill(&point!( 0.1, -0.1)); } // bin 1
        for _ in 0..3 { hist.fill(&point!(-0.1,  0.1)); } // bin 2
        for _ in 0..4 { hist.fill(&point!( 0.1,  0.1)); } // bin 3
        let m = hist.data();
        let expected = vec![1, 2, 3, 4];
        m.iter()
         .zip(expected.iter())
         .for_each(|(got, exp)| {
            assert_eq!(got, exp);
        });
    }
}
