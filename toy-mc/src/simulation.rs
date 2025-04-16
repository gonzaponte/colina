use std::f64::consts::{PI, TAU};
use nalgebra::{point, Point2, Point3, vector};

use crate::random::{uniform, poisson, normal, random_in_circle};

pub fn generate_el_position(el_r: f64) -> Point2<f64> {
    random_in_circle(el_r)
}

pub fn generate_electrons(p0: Point2<f64>, n_ave: f64, fano_factor: f64, cloud_r: f64) -> Vec<Point2<f64>> {
    let n = if n_ave < 10.0 { poisson(n_ave) as usize }
    else { normal(n_ave, n_ave.sqrt() * fano_factor).round() as usize };

    let p0 = p0 - Point2::origin();
    (0..n).into_iter()
          .map(|_| random_in_circle(cloud_r) + p0)
          .collect()
}

pub fn propagate_to_wire(p0: Point2<f64>, wire_pitch: f64, first_wire: f64, wire_r: f64, el_range: f64) -> (Point3<f64>, usize) {
    let dx       = p0.x - (first_wire - wire_pitch/2.);
    let n_wire   = (dx / wire_pitch).floor();
    let wire_pos = n_wire * wire_pitch + first_wire;
    let dx       = p0.x - wire_pos;
    let phi      = PI * (dx/wire_pitch - 0.5);
    let dist     = uniform(0., el_range) + wire_r;
    let x        = dist * phi.cos();
    let z        = dist * phi.sin();
    (point!(x + wire_pos, p0.y, z), n_wire as usize)
}

fn is_shadowed(p0: &Point3<f64>, pwire: &Point3<f64>, wire_r: f64, cos_th: f64, phi: f64) -> bool {
    let sin_th = (1.0 - cos_th.powi(2)).sqrt();
    let axis   = vector!(    pwire.x - p0.x, pwire.z - p0.z);
    let ray    = vector!(sin_th * phi.cos(),         cos_th);

    let a =  ray.dot(&ray );
    let b =  ray.dot(&axis); // negative sign irrelevant, factor 2 factored out
    let c = axis.dot(&axis) - wire_r*wire_r;
    b*b >= a*c
}

pub fn propagate_light(p0: Point3<f64>, pwire: Point3<f64>, light_yield: f64, distance: f64, wire_r: f64) -> Vec<Point2<f64>> {
    let light_yield = light_yield / 2.0;
     // TODO: consider CP factor
    // let n = normal(light_yield, light_yield.sqrt() * cp_factor).round() as usize;
    let n = poisson(light_yield) as usize;

    (0..n).into_iter()
        .map   (|_| (uniform(0.0, 1.0), uniform(0.0, TAU)))
        .filter(|(cos_th, phi)| !is_shadowed(&p0, &pwire, wire_r, *cos_th, *phi))
        .map   (|(cos_th, phi)| {
            let theta = cos_th.acos();
            let z     = distance - p0.z; // p0.z is negative
            let x     = z * theta.tan() * phi.cos();
            let y     = x               * phi.tan();
            point!(x + p0.x, y + p0.y)
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::f64::consts::{FRAC_PI_2, PI};
    use crate::random::uniform;

    #[test]
    fn generation_within_el() {
        let r = 0.123;
        for _ in 0..10_000 {
            let p0 = generate_el_position(r) - Point2::origin();
            assert!(p0.norm() < r);
        }
    }

    #[test]
    fn generation_within_cloud() {
        let p0      = point!(12.3, 45.6);
        let n       = 10_000_f64;
        let fano    = 0.0;
        let cloud_r = 7.89;
        let ps = generate_electrons(p0, n, fano, cloud_r);
        for p in ps {
            assert!((p - p0).norm() < cloud_r);
        }
    }

    #[test]
    fn mapping_to_wire() {
        let wire_pitch =  2.0;
        let first_wire = -1.0;
        let wire_r     =  0.5;
        let el_range   =  0.1; // irrelevant
        for _ in 0..10_000 {
            let x  = uniform(0.0, wire_pitch) + first_wire;
            let p0 = point!(x, 0.0);
            let (p1, iw) = propagate_to_wire(p0, wire_pitch, first_wire, wire_r, el_range);

            let expected_w = if x.is_sign_negative() {0} else {1};
            assert_eq!(iw, expected_w);
            assert_eq!(p1.x.is_sign_positive(), p0.x.is_sign_positive());
        }
    }

    #[test]
    fn shadow_phi_cases() {
        let p0     = point!(0.0, 0.0, -1.0);
        let p1     = point!(0.0, 0.0, 0.0);
        let r      = 1e-3;
        let cos_th = 0.5;
        let phis   = [0.0, FRAC_PI_2, PI, 3.0*FRAC_PI_2];
        let shadow = [false, true, false, true];
        for i in 0..4 {
            let outcome = is_shadowed(&p0, &p1, r, cos_th, phis[i]);
            assert_eq!(outcome, shadow[i], "failed for case {i}: phi:{} pi", phis[i]/PI);
        }
    }

    #[test]
    fn shadow_onaxis() {
        let r      = 1.0;
        let phi    = 0.0;
        let cos_th = 1.0;
        let p1     = point!(0.0, 0.0, 0.0);
        for _ in 0..10_000 {
            let x  = uniform(-  3.0*r,   3.0*r);
            let y  = uniform(-100.0*r, 100.0*r);
            let p0 = point!(x, y, -2.0*r);

            let outcome  = is_shadowed(&p0, &p1, r, cos_th, phi);
            let expected = x.abs() < r;
            assert_eq!(outcome, expected, "failed for x {}", x);
        }
    }

    #[test]
    fn shadow_approx() {
        let p0  = point!(0.0, 0.0, -2.0);
        let p1  = point!(0.0, 0.0, 0.0);
        let r   = 1.0;
        let phi = 0.0;

        for _ in 0..10_000 {
            let cos_th   = uniform(0.0, 1.0);
            let sin_th   = (1.0 - cos_th.powi(2)).sqrt();
            let outcome  = is_shadowed(&p0, &p1, r, cos_th, phi);
            let expected = sin_th < 0.5;
            assert_eq!(outcome, expected, "failed for theta {} pi, cos {}", cos_th.acos() / PI, cos_th);
        }
    }

}
