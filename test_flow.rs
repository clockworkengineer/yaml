use std::fs;
use std::path::Path;

fn main() {
    let tests = vec![
        ("9C9N", "Wrong indented flow sequence"),
        ("C2SP", "Flow Mapping Key on two lines"),
        ("G5U8", "Plain dashes in flow sequence"),
        ("KS4U", "Invalid item after end of flow sequence"),
        ("N782", "Invalid document markers in flow style"),
        ("T833", "Flow mapping missing a separating comma"),
        ("YJV2", "Dash in flow sequence"),
    ];

    for (id, desc) in tests {
        println!("\n=== {} - {} ===", id, desc);

        let test_dir = format!("tests/yaml-test-suite/{}/", id);
        let in_path = format!("{}in.yaml", test_dir);

        if Path::new(&in_path).exists() {
            let content = fs::read_to_string(&in_path).unwrap();
            println!("Input:\n{}", content);

            match yaml_rust_ww::parse(&content) {
                Ok(_) => println!("Result: PARSED (but should ERROR)"),
                Err(e) => println!("Result: ERROR - {}", e),
            }
        }
    }

    // Also test the "00" case
    println!("\n=== 00 - Flow collections over many lines ===");
    let content = "k: {\nk\n:\nv\n}";
    println!("Input:\n{}", content);
    match yaml_rust_ww::parse(&content) {
        Ok(_) => println!("Result: PARSED (but should ERROR)"),
        Err(e) => println!("Result: ERROR - {}", e),
    }
}
