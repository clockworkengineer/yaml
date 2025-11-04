
use std::path::Path;
// Import the necessary types and functions from yaml_lib and yaml_utility_lib
use yaml_lib::{FileSource, parse, FileDestination, to_bencode};
use yaml_utility_lib::get_yaml_file_list;

/// Processes a single yaml file by converting it to XML format.
///
/// # Arguments
/// * `file_path` - Path to the yaml file to be converted
///
/// # Returns
/// * `Result<(), String>` - Ok(()) if successful, Err with an error message if failed
fn process_yaml_file(file_path: &str) -> Result<(), String> {
    // Create a new file source for reading yaml data
    let mut source = FileSource::new(file_path).map_err(|e| e.to_string())?;

    // Parse the yaml content into a Node structure
    let node = parse(&mut source).map_err(|e| e.to_string())?;

    // Create a new file destination with .xml extension for the output
    let mut destination = FileDestination::new(
        Path::new(file_path)
            .with_extension("ben")
            .to_string_lossy()
            .as_ref()
    ).map_err(|e| e.to_string())?;

    // Convert the parsed yaml node to XML format and write to destination
    to_bencode(&node, &mut destination).unwrap();
    Ok(())
}

fn main() {
    // Get a list of all yaml files in the "files" directory
    let yaml_files = get_yaml_file_list("files");

    // Process each yaml file in the list
    for file_path in yaml_files {
        // Attempt to convert each file and handle any errors
        match process_yaml_file(&file_path) {
            Ok(()) => println!("Successfully converted {}", file_path),
            Err(e) => eprintln!("Failed to convert {}: {}", file_path, e),
        }
    }
}
