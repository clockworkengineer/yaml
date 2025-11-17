use yaml_lib::{parse_yaml_string};

fn main() {
    let yaml = "---\nkey: value\n... invalid\n";
    match parse_yaml_string(yaml) {
        Ok(doc) => println!("SUCCESS (should have failed!): {:?}", doc),
        Err(e) => println!("ERROR (correct!): {}", e),
    }
}
