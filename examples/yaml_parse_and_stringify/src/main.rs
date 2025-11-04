//! Module: examples/yaml_parse_and_stringify/src/main.rs

use std::path::Path;
use yaml_lib::{FileDestination, FileSource, parse, stringify};
use yaml_utility_lib::get_yaml_file_list;

/// Processes a single yaml file by converting it to bencode format
///
/// # Arguments
/// * `file_path` - Path to the yaml file to be processed
///
/// # Returns
/// * `Result<(), String>` - Ok(()) if successful, Err with an error message if failed
fn process_yaml_file(file_path: &str) -> Result<(), String> {

    let mut source = FileSource::new(file_path).map_err(|e| e.to_string())?;


    let node = parse(&mut source).map_err(|e| e.to_string())?;


    let mut destination = FileDestination::new(
        Path::new(file_path)
            .with_extension("yaml.stringify")
            .to_string_lossy()
            .as_ref(),
    )
    .map_err(|e| e.to_string())?;


    stringify(&node, &mut destination)?;
    Ok(())
}

fn main() {

    let yaml_files = get_yaml_file_list("files");


    for file_path in yaml_files {
        match process_yaml_file(&file_path) {
            Ok(()) => println!("Successfully converted {}", file_path),
            Err(e) => eprintln!("Failed to convert {}: {}", file_path, e),
        }
    }
}
