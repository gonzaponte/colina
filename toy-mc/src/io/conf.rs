use std::io;
use std::io::Write;
use std::fs::File;

use toml;

use crate::SimConfig;

pub fn write_conf(filename: &str, conf: &SimConfig) -> io::Result<()> {
    let mut file = File::create(filename)?;
    let contents = toml::to_string(conf).expect("Could not serialize config");
    file.write_all(contents.as_bytes())
}
