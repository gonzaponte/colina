use std::io;
use std::path::Path;
use std::fs::{create_dir_all, File};
use nalgebra::{point, Rotation2, DMatrix};
use clap::Parser;

use toymc::SimConfig;
use toymc::io::write_img_1d;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {

    #[arg(short, long)]
    conf: String,

    #[arg(short, long)]
    output: Option<String>,

}

fn main() -> io::Result<()> {
    let args = CLI::parse();
    let conf = SimConfig::new(&args.conf).unwrap().overrides(None, args.output);
    let path = Path::new(&conf.output);
    if !path.exists() { create_dir_all(path).expect("Could not create directoty"); }

    let filename_img = path.join("picture.csv").to_str().unwrap().to_owned();

    let wires      = &conf.geometry.wire_plane;
    let sipms      = &conf.geometry.sipm_plane;
    let first_wire = wires.wire_pos().first().unwrap().clone();
    let rotation   = Rotation2::new(wires.wire_rotation);
    let sipm_bins  = sipms.sipm_bins();

    let edge = sipm_bins.first().unwrap();
    let n    = 2001;
    let fine_bins : Vec<f64> =
        (0..=n).into_iter()
        .map(|i| edge - (i as f64 / n as f64)*2.0*edge)
        .collect();

    let xybins = fine_bins[0..n-1].iter()
                                  .zip(fine_bins[1..].iter())
                                  .map(|(l,r)| (l + r)/2.)
                                  .collect::<Vec<_>>();

    let pitch   = sipms.sipm_pitch();
    let size    = sipms.sipm_size;
    let mut img = DMatrix::<usize>::zeros(n-1, n-1);
    for (i, x) in xybins.iter().enumerate() {
        for (j, y) in xybins.iter().enumerate() {
            let mut ok = true;

            let p  = point!(*x, *y);
            let dx = (p.x - sipm_bins[0]).rem_euclid(pitch) - pitch/2.;
            let dy = (p.y - sipm_bins[0]).rem_euclid(pitch) - pitch/2.;
            ok = ok && dx.abs() < size/2.;
            ok = ok && dy.abs() < size/2.;

            let p  = rotation * p;
            let dx = (p.x - first_wire - wires.wire_r).rem_euclid(wires.wire_pitch);
            ok = ok && dx.abs() > wires.wire_r*2.;

            if ok {
                img[(i,j)] = 1;
            }
        }
    }

    let m = img.transpose();
    let mut file = File::create(filename_img)?;
    write_img_1d(&mut file, &m)?;

    Ok(())
}
