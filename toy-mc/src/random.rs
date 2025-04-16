use std::f64::consts::TAU;
use nalgebra::{point, Point2};
use rand::rng;
use rand_distr::{Poisson, Normal, Uniform, Distribution};


pub fn random_in_circle(r: f64) -> Point2<f64> {
    let r   = uniform(0.0, r.powi(2)).sqrt();
    let phi = uniform(0.0, TAU);
    point!(r * phi.cos(), r * phi.sin())
}


pub fn uniform(low: f64, high:f64 ) -> f64 { Uniform::new(low, high).unwrap().sample(&mut rng()) }
pub fn poisson(mean: f64          ) -> f64 { Poisson::new(mean     ).unwrap().sample(&mut rng()) }
pub fn normal (mean: f64, std: f64) -> f64 { Normal ::new(mean, std).unwrap().sample(&mut rng()) }


#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn uniform_within_range() {
        let l = 123.4;
        let h = 567.8;
        for _ in 0..1_000 {
            let x = uniform(l, h);
            assert!(x>l);
            assert!(x<h);
        }
    }

    #[test]
    fn poisson_int() {
        let mean = 1.23;
        for _ in 0..1_000 {
            let x = poisson(mean);
            assert_float_eq!(x.floor(), x, ulps<=2);
        }
    }

    #[test]
    fn poisson_zero_or_positive() {
        let mean = 0.123;
        for _ in 0..1_000 {
            let x = poisson(mean);
            assert!(x >= 0.0);
        }
    }

    #[test]
    fn circle_within_r() {
        let r = 123.4;
        for _ in 0..1_000 {
            let p = random_in_circle(r) - Point2::origin();
            assert!(p.magnitude() < r);
        }
    }
}
