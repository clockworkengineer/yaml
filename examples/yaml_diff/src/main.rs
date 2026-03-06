//! Example demonstrating YAML document diffing.
//!
//! Uses `diff_nodes()` to compare two YAML documents and categorise every
//! change as an addition, removal, modification, type change, or size change.
//!
//! Useful for auditing config changes, testing round-trips, and building
//! human-readable "what changed?" reports.

use yaml_lib::{Diff, DiffType, diff_nodes, parse_string};

// --- sample documents ---------------------------------------------------------

const V1: &str = r#"name: Alice
age: 30
role: developer
skills:
  - Rust
  - Go
address:
  city: London
  country: UK
"#;

const V2: &str = r#"name: Alice
age: 31
role: senior developer
skills:
  - Rust
  - Python
email: alice@example.com
address:
  city: London
  country: United Kingdom
"#;

const V3: &str = r#"database:
  host: db.prod.internal
  port: 5432
  pool_size: 20
"#;

// --- helpers ------------------------------------------------------------------

fn print_diff(diff: &Diff) {
    let symbol = match diff.diff_type {
        DiffType::Added => "+",
        DiffType::Removed => "-",
        DiffType::Modified | DiffType::TypeChanged => "~",
        DiffType::SizeChanged => "#",
    };

    print!("  {} [{}] {}", symbol, diff.path, diff.description);

    match (&diff.old_value, &diff.new_value) {
        (Some(old), Some(new)) => println!("  ({} → {})", old, new),
        (Some(old), None) => println!("  (was: {})", old),
        (None, Some(new)) => println!("  (now: {})", new),
        (None, None) => println!(),
    }
}

// --- main ---------------------------------------------------------------------

fn main() {
    println!("=== YAML Diff Example ===\n");

    let v1 = parse_string(V1).expect("parse V1");
    let v2 = parse_string(V2).expect("parse V2");
    let v3 = parse_string(V3).expect("parse V3");

    // ------------------------------------------------------------------
    // 1. Compare identical documents — should report zero differences
    // ------------------------------------------------------------------
    println!("--- 1. Comparing a document with itself ---");
    let same = diff_nodes(&v1, &v1);
    println!("  Identical : {}", same.is_identical());
    println!("  Differences: {}\n", same.count());

    // ------------------------------------------------------------------
    // 2. Compare V1 vs V2 (modified config)
    // ------------------------------------------------------------------
    println!("--- 2. V1 vs V2 (config update) ---");
    let result = diff_nodes(&v1, &v2);
    println!(
        "  Identical : {}   Differences: {}",
        result.is_identical(),
        result.count()
    );
    for diff in &result.diffs {
        print_diff(diff);
    }
    println!();

    // ------------------------------------------------------------------
    // 3. Filter by diff type
    // ------------------------------------------------------------------
    println!("--- 3. Additions only ---");
    for diff in result
        .diffs
        .iter()
        .filter(|d| d.diff_type == DiffType::Added)
    {
        print_diff(diff);
    }

    println!("\n--- 3. Modifications only ---");
    for diff in result
        .diffs
        .iter()
        .filter(|d| d.diff_type == DiffType::Modified)
    {
        print_diff(diff);
    }

    println!("\n--- 3. Removals only ---");
    for diff in result
        .diffs
        .iter()
        .filter(|d| d.diff_type == DiffType::Removed)
    {
        print_diff(diff);
    }
    println!();

    // ------------------------------------------------------------------
    // 4. Use DiffResult::format() for a single ready-made report string
    // ------------------------------------------------------------------
    println!("--- 4. DiffResult::format() ---");
    println!("{}", result.format());

    // ------------------------------------------------------------------
    // 5. Completely different documents
    // ------------------------------------------------------------------
    println!("--- 5. Completely different documents ---");
    let diff_v1_v3 = diff_nodes(&v1, &v3);
    println!(
        "  Identical : {}   Differences: {}",
        diff_v1_v3.is_identical(),
        diff_v1_v3.count()
    );
    for diff in &diff_v1_v3.diffs {
        print_diff(diff);
    }
    println!();

    // ------------------------------------------------------------------
    // 6. Round-trip check: parse → stringify → re-parse should be identical
    // ------------------------------------------------------------------
    println!("--- 6. Round-trip stability check ---");
    use yaml_lib::{BufferDestination, stringify};
    let mut buf = BufferDestination::new();
    stringify(&v1, &mut buf).expect("stringify");
    let reparsed = parse_string(&buf.to_string()).expect("re-parse");
    let rt_diff = diff_nodes(&v1, &reparsed);
    if rt_diff.is_identical() {
        println!("  ✓ Round-trip is stable — no differences after parse → stringify → parse");
    } else {
        println!(
            "  ✗ Round-trip changed the document ({} diffs):",
            rt_diff.count()
        );
        for diff in &rt_diff.diffs {
            print_diff(diff);
        }
    }
    println!();
}
