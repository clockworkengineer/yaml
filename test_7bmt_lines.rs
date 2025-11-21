use yaml_lib::{parse, BufferSource};

fn test(name: &str, yaml: &str) {
    println!("\n=== {} ===", name);
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(_) => println!("✓ OK"),
        Err(e) => println!("✗ {}", e),
    }
}

fn main() {
    // Build up 7BMT line by line
    test("Line 1-3", "---\ntop1: &node1\n  &k1 key1: one");
    test("Line 1-5", "---\ntop1: &node1\n  &k1 key1: one\ntop2: &node2 # comment\n  key2: two");
    test("Line 1-7", "---\ntop1: &node1\n  &k1 key1: one\ntop2: &node2 # comment\n  key2: two\ntop3:\n  &k3 key3: three");
    test("Line 1-9", "---\ntop1: &node1\n  &k1 key1: one\ntop2: &node2 # comment\n  key2: two\ntop3:\n  &k3 key3: three\ntop4: &node4\n  &k4 key4: four");
    test("Line 1-11", "---\ntop1: &node1\n  &k1 key1: one\ntop2: &node2 # comment\n  key2: two\ntop3:\n  &k3 key3: three\ntop4: &node4\n  &k4 key4: four\ntop5: &node5\n  key5: five");
    test("Line 1-13", "---\ntop1: &node1\n  &k1 key1: one\ntop2: &node2 # comment\n  key2: two\ntop3:\n  &k3 key3: three\ntop4: &node4\n  &k4 key4: four\ntop5: &node5\n  key5: five\ntop6: &val6\n  six");
    test("Full 7BMT", "---\ntop1: &node1\n  &k1 key1: one\ntop2: &node2 # comment\n  key2: two\ntop3:\n  &k3 key3: three\ntop4: &node4\n  &k4 key4: four\ntop5: &node5\n  key5: five\ntop6: &val6\n  six\ntop7:\n  &val7 seven");
}
