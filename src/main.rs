use std::env;
use std::fs::File;
use std::fs::write;
use std::path::Path;
use safetensors::tensor::SafeTensors;
use memmap2::Mmap;
use serde_json::{ json, Value };
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename or directory>", args[0]);
        return Ok(());
    }
    let path = &args[1];
    if Path::new(path).is_dir() {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if
                entry
                    .path()
                    .extension()
                    .and_then(|s| s.to_str()) == Some("safetensors")
            {
                process_file(entry.path())?;
            }
        }
    } else {
        process_file(Path::new(path))?;
    }
    Ok(())
}
fn process_file(path: &Path) -> std::io::Result<()> {
    let file = File::open(path)?;
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
        let output_filename = path
            .parent()
            .unwrap()
            .join(format!("{}.json", path.file_stem().unwrap().to_str().unwrap()));
        write(output_filename, pretty_json)?;
    }
    Ok(())
}
