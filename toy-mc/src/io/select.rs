use std::io;
use clap::ValueEnum;

use crate::{Event, SimConfig};
use crate::io::csv    ::get_writer as     csv_writer;
use crate::io::feather::get_writer as feather_writer;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Writer {
    Csv,
    Feather,
}

pub fn writer(filename: &str, format: Writer, conf: &SimConfig) -> Box<dyn FnMut(&Event) -> io::Result<()>> {
    match format {
        Writer::Csv     =>     csv_writer(filename, conf),
        Writer::Feather => feather_writer(filename, conf),
    }
}
