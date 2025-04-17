use std::fs::File;
use std::io;
use std::io::Write;
use nalgebra::DMatrix;
use nalgebra::RowDVector;
use itertools::Itertools;

use crate::{Event, SimConfig};

fn _row_as_str(row: RowDVector<usize>) -> String {
    #[allow(unstable_name_collisions)]
    row.iter()
        .map(usize::to_string)
        .intersperse(" ".to_owned())
        .collect()
}

fn _img_as_str_2d(img: &DMatrix<usize>) -> String
{
    #[allow(unstable_name_collisions)]
    let mut s : String =
        img.row_iter()
           .map(|r| _row_as_str(r.into()))
           .intersperse("\n".to_owned())
           .collect();
    s.push('\n');
    s
}

fn img_as_str_1d(img: &DMatrix<usize>) -> String {
    #[allow(unstable_name_collisions)]
    img.transpose()
       .iter()
       .map(usize::to_string)
       .intersperse(" ".to_owned())
       .collect()
}

fn vec_as_str<T: ToString>(vec: &Vec<T>) -> String {
    #[allow(unstable_name_collisions)]
    vec.iter()
       .map(T::to_string)
       .intersperse(" ".to_owned())
       .collect()
}

fn _write_img_2d(file: &mut File, img: &DMatrix<usize>) -> io::Result<()> {
    let contents = _img_as_str_2d(img);
    file.write_all(contents.as_bytes())
}

pub fn write_img_1d(file: &mut File, img: &DMatrix<usize>) -> io::Result<()> {
    let contents = img_as_str_1d(img) + "\n";
    file.write_all(contents.as_bytes())
}

pub fn write_header(file: &mut File, n_wires: usize, img_size: usize) -> io::Result<()> {
    let mut line = String::new();
    line.push_str("event x0 y0");
    (0..n_wires).into_iter().for_each(|w| line.push_str(&format!(" w_{}", w)));
    (0..img_size).into_iter()
                 .flat_map(|i| (0..img_size).into_iter().map(move |j| (i,j)))
                 .for_each(|(i,j)| line.push_str(&format!(" img_{}_{}", i, j)));
    line.push('\n');
    file.write_all(line.as_bytes())
}

fn write_event(file: &mut File, event: &Event) -> io::Result<()> {
    let mut line = String::new();
    line.push_str(&event.number    .to_string()); line.push(' ');
    line.push_str(&event.position.x.to_string()); line.push(' ');
    line.push_str(&event.position.y.to_string()); line.push(' ');
    line.push_str(&vec_as_str(&event.wire_q)   ); line.push(' ');
    line.push_str(&img_as_str_1d(&event.img)   ); line.push('\n');
    file.write_all(line.as_bytes())
}

pub fn get_writer(filename: &str, conf: &SimConfig) -> Box<dyn FnMut(&Event) -> io::Result<()>> {
    let mut file = File::create(filename).unwrap();
    write_header(&mut file, conf.geometry.wire_plane.n_wires, conf.geometry.sipm_plane.n_sipms_side).unwrap();
    Box::new( move |e: &Event| {
        write_event(&mut file, e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempfile;
    use std::io::{Read, Seek};
    use nalgebra::point;

    #[test]
    fn row() {
        let m = DMatrix::from_vec(1, 3, vec![1usize, 2, 3]);
        let r = m.row(0);
        let s = _row_as_str(r.into());
        assert_eq!(s, "1 2 3".to_owned());
    }

    #[test]
    fn img_2d() {
        let m = DMatrix::from_vec(2, 3, vec![1usize, 4, 2, 5, 3, 6]);
        let s = _img_as_str_2d(&m);
        assert_eq!(s, "1 2 3\n4 5 6\n".to_owned());
    }

    #[test]
    fn img_1d() {
        let m = DMatrix::from_vec(2, 3, vec![1usize, 4, 2, 5, 3, 6]);
        let s = img_as_str_1d(&m);
        assert_eq!(s, "1 2 3 4 5 6".to_owned());
    }

    #[test]
    fn img_2d_write() {
        let m        = DMatrix::from_vec(2, 3, vec![1usize, 4, 2, 5, 3, 6]);
        let mut file = tempfile().unwrap();
        _write_img_2d(&mut file, &m).unwrap();
        file.seek(io::SeekFrom::Start(0)).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!("1 2 3\n4 5 6\n", buffer);
    }

    #[test]
    fn img_1d_write() {
        let m        = DMatrix::from_vec(2, 3, vec![1usize, 4, 2, 5, 3, 6]);
        let mut file = tempfile().unwrap();
        write_img_1d(&mut file, &m).unwrap();
        file.seek(io::SeekFrom::Start(0)).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!("1 2 3 4 5 6\n", buffer);
    }

    #[test]
    fn header_write() {
        let mut file = tempfile().unwrap();
        write_header(&mut file, 3, 2).unwrap();
        file.seek(io::SeekFrom::Start(0)).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!("event x0 y0 w_0 w_1 w_2 img_0_0 img_0_1 img_1_0 img_1_1\n", buffer);
    }

    #[test]
    fn event_write() {
        let e = Event{
            number: 123,
            position: point!(4.56, 7.89),
            wire_q: vec![3, 1, 4, 15, 92, 65, 35, 89, 79],
            img: DMatrix::from_vec(2, 2, vec![1usize, 100, 10, 1000]),
        };
        let mut file = tempfile().unwrap();
        write_event(&mut file, &e).unwrap();
        file.seek(io::SeekFrom::Start(0)).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!("123 4.56 7.89 3 1 4 15 92 65 35 89 79 1 10 100 1000\n", buffer);
    }

    #[test]
    fn stupid() {
        let m = DMatrix::from_vec(2, 2, vec![1usize, 100, 10, 1000]);
        assert_eq!(m[(0,0)], 1);
        assert_eq!(m[(0,1)], 10);
        assert_eq!(m[(1,0)], 100);
        assert_eq!(m[(1,1)], 1000);
        let v : Vec<usize> = m.iter().cloned().collect();
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 100);
        assert_eq!(v[2], 10);
        assert_eq!(v[3], 1000);
    }
}
