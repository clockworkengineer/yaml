use yaml_lib::{BufferSource, parse};

fn main() {
    println!("Testing MUS6/02 (%YAML 1.1 with ---):");
    let yaml = b"%YAML  1.1\n---\n";
    let mut source = BufferSource::new(yaml);
    match parse(&mut source) {
        Ok(docs) => {
            println!("  Parse succeeded: {:#?}", docs);
        }
        Err(e) => {
            println!("  Parse failed: {}", e);
        }
    }
    
    println!("\nTesting 9MMA (%YAML 1.2 without ---):");
    let yaml2 = b"%YAML 1.2\n";
    let mut source2 = BufferSource::new(yaml2);
    match parse(&mut source2) {
        Ok(docs) => {
            println!("  Parse succeeded: {:#?}", docs);
        }
        Err(e) => {
            println!("  Parse failed: {}", e);
        }
    }
}
