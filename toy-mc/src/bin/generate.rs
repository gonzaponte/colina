use std::io;
use std::path::Path;
use std::fs::{create_dir, File};
use nalgebra::{point, Point2, Rotation2, DMatrix};
use indicatif::ProgressBar;
use clap::Parser;

use toymc::{Image, Event, SimConfig};
use toymc::io::write_conf;
use toymc::io::{writer, write_img_1d, Writer};
use toymc::simulation::{generate_el_position, generate_electrons, propagate_to_wire, propagate_light};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {

    #[arg(short, long)]
    conf: String,

    #[arg(short, long)]
    nevt: Option<usize>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long, value_enum, default_value_t=Writer::Csv)]
    format: Writer,

    #[arg(long, action)]
    detailed: bool,
}

fn main() -> io::Result<()> {
    let args = CLI::parse();
    let conf = SimConfig::new(&args.conf)
                         .unwrap()
                         .overrides(args.nevt, args.output);
    let path = Path::new(&conf.output);
    if !path.exists() { create_dir(path)?; }

    let filename_img  = "images.".to_string() + match args.format {
        Writer::Csv     =>     "csv",
        Writer::Feather => "feather",
    };
    let filename_img  = path.join(        &filename_img).to_str().unwrap().to_owned();
    let filename_conf = path.join(           "run.conf").to_str().unwrap().to_owned();
    let filename_fine = path.join("detailed_images.csv").to_str().unwrap().to_owned();

    let wires      = &conf.geometry.wire_plane;
    let sipms      = &conf.geometry.sipm_plane;
    let elgap      = &conf.geometry.el_gap;
    let params     = &conf.sim_params;
    let all_wires  = wires.wire_pos();
    let first_wire = all_wires.first().unwrap().clone();
    let rotation   = Rotation2::new(-wires.wire_rotation);
    let sipm_bins  = sipms.sipm_bins();

    write_conf(&filename_conf, &conf)?;
    let mut write_event = writer(&filename_img, args.format, &conf);
    let mut file_fine   = if args.detailed { Some(File::create(filename_fine)?) } else { None };


    let edge   = sipm_bins.first().unwrap();
    let n_fine = 100;
    let fine_bins : Vec<f64> =
        (0..=n_fine).into_iter()
        .map(|i| edge - (i as f64 / n_fine as f64)*2.0*edge)
        .collect();

    let bar      = ProgressBar::new(conf.n_events as u64);
    let flushmod = (conf.n_events / 100).max(1);
    for ievt in 0..conf.n_events {
        if ievt.rem_euclid(flushmod) == 0 {
            bar.inc(flushmod as u64);
        }

        let mut img      = Image::new(&sipms.sipm_bins());
        let mut img_fine = if args.detailed { Some(Image::new(&fine_bins)) } else { None };
        let mut wire_q   = vec![0usize; wires.n_wires];
        let evt_pos      = generate_el_position(elgap.el_r);
        let ps           = generate_electrons(evt_pos, params.n_ie_ave(), params.fano_factor, params.cloud_r);
        for p0 in ps {
            let (p1, iwire) = propagate_to_wire(p0, wires.wire_pitch, first_wire, wires.wire_r, params.el_range);
            wire_q[iwire] += 1;
            let wire = all_wires.get(iwire).unwrap();
            let wire = point!(*wire, p0.y, 0.0);
            let hits = propagate_light(p1, wire, params.light_yield, conf.geometry.buffer, wires.wire_r);
            let hits = hits.iter().map(|h| rotation * h).collect::<Vec<Point2<_>>>();
            for hit in &hits {
                img.fill(hit);
                if args.detailed { img_fine.as_mut().unwrap().fill(hit); }
            }
        }
        let event = Event{number: ievt, position: evt_pos, wire_q, img: img.finalize()};
        write_event(&event)?;

        if args.detailed {
            let n = fine_bins.len() - 1;
            let m = DMatrix::from_vec(n, n, img_fine.unwrap().data()).transpose();
            write_img_1d(file_fine.as_mut().unwrap(), &m)?;
        }
    }
    bar.finish();

    Ok(())
}
