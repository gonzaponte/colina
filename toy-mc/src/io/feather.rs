use std::fs::File;
use std::sync::Arc;
use std::io;

use arrow::array::{UInt32Array, Float32Array, ArrayRef};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow::ipc::writer::FileWriter;

use crate::{Event, SimConfig};


pub fn generate_schema(n_wires: usize, n_sipms: usize) -> Arc<Schema> {
    let mut fields = vec![
        Field::new("event", DataType::UInt32 , false),
        Field::new(    "x", DataType::Float32, false),
        Field::new(    "y", DataType::Float32, false),
    ];
    for i in 0..n_wires {
        let name = format!("wire_{i}");
        fields.push(Field::new(name, DataType::UInt32, false));
    }
    for i in 0..n_sipms {
        for j in 0..n_sipms {
            let name = format!("img_{i}_{j}");
            fields.push(Field::new(name, DataType::UInt32, false))
        }
    }
    Arc::new(Schema::new(fields))
}

fn create_record_batch(e: &Event, s: Arc<Schema>) -> RecordBatch {
    let mut fields : Vec<ArrayRef> = Vec::new();
    fields.push(Arc::new( UInt32Array::from(vec![e.number     as u32])));
    fields.push(Arc::new(Float32Array::from(vec![e.position.x as f32])));
    fields.push(Arc::new(Float32Array::from(vec![e.position.y as f32])));
    for q in &e.wire_q { fields.push(Arc::new(UInt32Array::from(vec![*q as u32]))); }
    for q in &e.img    { fields.push(Arc::new(UInt32Array::from(vec![*q as u32]))); }
    RecordBatch::try_new(s.clone(), fields).unwrap()
}

pub fn get_writer(filename: &str, conf: &SimConfig) -> Box<dyn FnMut(&Event) -> io::Result<()>> {
    let     schema = generate_schema(conf.geometry.wire_plane.n_wires, conf.geometry.sipm_plane.n_sipms_side);
    let     file   = File::create(filename).unwrap();
    let mut writer = FileWriter::try_new(file, &schema).unwrap();
    Box::new( move |e: &Event| {
        let rb = create_record_batch(e, schema.clone());
        let ok = writer.write(&rb).unwrap();
        Ok(ok)
    })
}
