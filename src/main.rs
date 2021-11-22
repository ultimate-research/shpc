use shpc_lib::Shan;
use ssbh_write::SsbhWrite;
use std::env;
use std::io::{BufReader, BufWriter};

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = std::path::Path::new(&args[1]);
    match input.extension().unwrap().to_str().unwrap() {
        "json" => {
            let json_file = BufReader::new(std::fs::File::open(input).unwrap());
            let shan_file: Shan = serde_json::from_reader(json_file).unwrap();

            let mut output = BufWriter::new(std::fs::File::create(&args[2]).unwrap());
            shan_file.write(&mut output).unwrap();
        }
        _ => {
            let shan_file = shpc_lib::read_shan_file(&args[1]);

            let output = BufWriter::new(std::fs::File::create(&args[2]).unwrap());
            serde_json::to_writer_pretty(output, &shan_file).unwrap();
        }
    }
}
