//! Example demonstrating the developer-tools module of yaml_lib.
//!
//! Shows how to use:
//! - `print_tree()` — visualise the full node tree as indented text
//! - `node_summary()` / `node_depth()` / `node_size()` / `node_type()` — quick per-node stats
//! - `NodeInfo` — rich, all-in-one metadata struct
//! - `find_by_type()` — collect every node of a given type from a tree
//! - `NodeDebugger` — log access / create / modify operations at configurable verbosity
//! - `DebugAssert` — development-time type, size, and depth assertions

use yaml_lib::{
    DebugAssert, NodeDebugger, NodeInfo, NodeType, find_by_type, node_depth, node_size,
    node_summary, node_type, parse_string, print_tree,
};

const YAML: &str = r#"# Application configuration
app:
  name: my-service
  version: "1.4.2"
  debug: false
  max_connections: 100
  tags:
    - web
    - api
    - rust
database:
  host: localhost
  port: 5432
  name: mydb
  credentials: &creds
    user: admin
    password: secret
replica:
  host: replica.internal
  credentials: *creds
"#;

fn main() {
    println!("=== YAML Developer Tools Example ===\n");

    let doc = parse_string(YAML).expect("parse failed");

    // ------------------------------------------------------------------
    // 1. Tree visualisation
    // ------------------------------------------------------------------
    println!("--- 1. Tree structure (print_tree) ---");
    let tree = print_tree(&doc);
    println!("{}", tree);

    // ------------------------------------------------------------------
    // 2. Root-level stats
    // ------------------------------------------------------------------
    println!("--- 2. Root-level stats ---");
    println!("  node_type   : {}", node_type(&doc).as_str());
    println!("  node_summary: {}", node_summary(&doc));
    println!("  node_depth  : {}", node_depth(&doc));
    println!("  node_size   : {}\n", node_size(&doc));

    // ------------------------------------------------------------------
    // 3. Rich NodeInfo struct
    // ------------------------------------------------------------------
    println!("--- 3. NodeInfo ---");
    let info = NodeInfo::new(&doc);
    println!("{}\n", info.format());

    // ------------------------------------------------------------------
    // 4. find_by_type — search the whole tree
    // ------------------------------------------------------------------
    println!("--- 4a. All string nodes ---");
    let strings = find_by_type(&doc, NodeType::String);
    println!("  Found {} string nodes:", strings.len());
    for s in &strings {
        println!("    {}", node_summary(s));
    }

    println!("\n--- 4b. All integer nodes ---");
    let integers = find_by_type(&doc, NodeType::Integer);
    println!("  Found {} integer nodes:", integers.len());
    for n in &integers {
        println!("    {}", node_summary(n));
    }

    println!("\n--- 4c. All boolean nodes ---");
    let bools = find_by_type(&doc, NodeType::Boolean);
    println!("  Found {} boolean nodes:", bools.len());
    for b in &bools {
        println!("    {}", node_summary(b));
    }

    println!("\n--- 4d. Anchored nodes ---");
    let anchored = find_by_type(&doc, NodeType::Anchored);
    println!("  Found {} anchored node(s)", anchored.len());
    for a in &anchored {
        println!("    {}", node_summary(a));
    }

    println!("\n--- 4e. Alias nodes ---");
    let aliases = find_by_type(&doc, NodeType::Alias);
    println!("  Found {} alias node(s)", aliases.len());
    for a in &aliases {
        println!("    {}", node_summary(a));
    }
    println!();

    // ------------------------------------------------------------------
    // 5. NodeDebugger — instrument access and mutation
    // ------------------------------------------------------------------
    println!("--- 5. NodeDebugger ---");
    let mut dbg = NodeDebugger::new();

    // Simulate accessing a path in the document
    dbg.debug_access("doc", &doc);
    dbg.debug_create(&doc);

    // Simulated before/after modification
    use yaml_lib::parse_string as ps;
    let old_val = ps("42").expect("parse old");
    let new_val = ps("43").expect("parse new");
    dbg.debug_modify("app.max_connections", &old_val, &new_val);

    println!("  Debug log:\n{}", dbg.logs());

    // ------------------------------------------------------------------
    // 6. DebugAssert — development-time structural assertions
    // ------------------------------------------------------------------
    println!("--- 6. DebugAssert ---");

    // Expect Documents node at root
    match DebugAssert::assert_type(&doc, NodeType::Documents) {
        Ok(()) => println!("  ✓ Root is a Documents node"),
        Err(e) => println!("  ✗ {}", e),
    }

    // Depth ≤ 10 — should pass easily
    match DebugAssert::assert_max_depth(&doc, 10) {
        Ok(()) => println!("  ✓ Depth ≤ 10"),
        Err(e) => println!("  ✗ {}", e),
    }

    // Intentionally failing size assertion to show the error message
    match DebugAssert::assert_size(&doc, 99) {
        Ok(()) => println!("  ✓ Size == 99 (unexpected)"),
        Err(e) => println!("  ✗ (expected failure) {}", e),
    }

    // Type-check individual nodes found earlier
    if let Some(first_str) = strings.first() {
        match DebugAssert::assert_type(first_str, NodeType::String) {
            Ok(()) => println!("  ✓ First string node is indeed a String"),
            Err(e) => println!("  ✗ {}", e),
        }
    }

    println!();
}
