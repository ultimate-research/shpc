use binrw::{BinReaderExt, BinWrite, WriteOptions};
use shpc_lib::Shan;
use ssbh_write::SsbhWrite;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    let args: Vec<String> = env::args().collect();
    let shan_file = shpc_lib::read_shan_file(&args[1]);
    // println!("{:#?}", shan_file);
    // println!("{}", serde_json::to_string_pretty(&shan_file).unwrap());
    let mut output = BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    shan_file.write(&mut output).unwrap();
}
