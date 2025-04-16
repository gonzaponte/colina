use nalgebra::{DMatrix, Point2};

pub struct Event {
    pub number  : usize,
    pub position: Point2<f64>,
    pub wire_q  : Vec<usize>,
    pub img     : DMatrix<usize>,
}
