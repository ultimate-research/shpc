use shpc::shan::Shan;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

// TODO: Create a higher level representation that handles coefficient compression.
fn parse_and_write_json<P: AsRef<Path>>(input: P, output: P) {
    let parse_start_time = Instant::now();
    match Shan::from_file(&input) {
        Ok(data) => {
            eprintln!("Parse: {:?}", parse_start_time.elapsed());

            let json = serde_json::to_string_pretty(&data).unwrap();

            let mut output_file = std::fs::File::create(output).expect("unable to create file");
            output_file
                .write_all(json.as_bytes())
                .expect("unable to write");
        }
        Err(error) => eprintln!("{:?}", error),
    };
}

fn deserialize_and_save(
    json: &str,
    input: &Path,
    output: &Option<PathBuf>,
    extension: &str,
) -> serde_json::Result<()> {
    let data = serde_json::from_str::<Shan>(json)?;

    let output_path = output
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(input).with_extension(extension));
    data.write_to_file(output_path).unwrap();
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("\tshpc_data_json <file>");
        eprintln!("\tshpc_data_json <file> <json output>");
        return;
    }

    let input = args.get(1).unwrap();
    let input_path = Path::new(&input);
    // Modify the input if no output is specified to allow dragging a file onto the executable.
    let output_path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(input.to_string() + ".json"));

    // Try parsing one of the supported formats.
    match input_path.extension().unwrap().to_str().unwrap() {
        "shpcanim" => parse_and_write_json(input_path, &output_path),
        "json" => {
            let json = std::fs::read_to_string(input_path).expect("Failed to read file.");
            let output_path = args.get(2).map(PathBuf::from);

            deserialize_and_save(&json, input_path, &output_path, "shpcanim").unwrap();
        }
        _ => (),
    };
}
