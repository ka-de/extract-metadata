use std::env;
use std::fs::File;
use std::fs::write;
use std::path::Path;
use safetensors::tensor::SafeTensors;
use memmap::Mmap;
use serde_json::{ json, Value };

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        return Ok(());
    }
    let filename = &args[1];
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let (_header_size, metadata) = SafeTensors::read_metadata(&mmap).unwrap();
    if let Some(json_metadata) = metadata.metadata() {
        let mut json: Value = json!(json_metadata);
        let json_fields = [
            "ss_bucket_info",
            "ss_tag_frequency",
            "ss_network_args",
            "ss_dataset_dirs",
        ];
        for field in &json_fields {
            if let Some(field_value) = json.get(*field).and_then(Value::as_str) {
                let parsed_field_value: Value = serde_json::from_str(field_value)?;
                *json.get_mut(*field).unwrap() = parsed_field_value;
            }
        }
        let pretty_json = serde_json::to_string_pretty(&json)?;
        println!("{}", pretty_json);
        let output_filename = Path::new(filename)
            .parent()
            .unwrap()
            .join(format!("{}.json", Path::new(filename).file_stem().unwrap().to_str().unwrap()));
        write(output_filename, pretty_json)?;
    }
    Ok(())
}
