use ssbh_write::SsbhWrite;
use std::env;
use std::io::BufWriter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let shan_file = shpc_lib::read_shan_file(&args[1]);
    println!("{:#?}", shan_file);
    // println!("{}", serde_json::to_string_pretty(&shan_file).unwrap());
    let mut output = BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    shan_file.write(&mut output).unwrap();
}
