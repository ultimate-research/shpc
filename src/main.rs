use binrw::BinReaderExt;
use shpc_lib::Shan;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let args: Vec<String> = env::args().collect();
    let shan_file = shpc_lib::read_shan_file(&args[1]);
    // println!("{:#?}", shan_file);
    println!("{}", serde_json::to_string_pretty(&shan_file).unwrap());
}
