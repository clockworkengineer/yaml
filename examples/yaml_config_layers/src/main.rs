//! Example demonstrating layered configuration with YAML.
//!
//! A common real-world pattern for 12-factor apps and microservices:
//!
//!   base config  (checked-in defaults)
//!       +
//!   environment overlay  (prod / staging / dev)
//!       +
//!   local override  (developer machine, never committed)
//!   ─────────────────────────────────────────────────
//!   effective config
//!
//! This example shows how to parse each layer as a YAML document and
//! recursively deep-merge them into a single `Node` tree, with later
//! layers winning on conflicts while preserving keys not present in
//! the overlay.

use yaml_lib::{parse_string, stringify, BufferDestination, Node};

// ---------------------------------------------------------------------------
// Config layers (in a real app these would come from files / env vars)
// ---------------------------------------------------------------------------

const BASE: &str = r#"app:
  name: my-service
  port: 8080
  debug: false
  log_level: warn
database:
  host: localhost
  port: 5432
  pool_size: 5
cache:
  enabled: true
  ttl_seconds: 300
"#;

/// Production: tighten up logging and scale the DB pool
const PROD: &str = r#"app:
  log_level: error
database:
  host: db.prod.internal
  pool_size: 20
"#;

/// Development: verbose logging, smaller pool, disable cache
const DEV: &str = r#"app:
  debug: true
  log_level: debug
database:
  pool_size: 2
cache:
  enabled: false
"#;

/// Local override: run on a different port to avoid clashing with other services
const LOCAL: &str = r#"app:
  port: 9090
"#;

// ---------------------------------------------------------------------------
// Deep-merge logic
// ---------------------------------------------------------------------------

/// Recursively merge `overlay` on top of `base`.
///
/// Rules:
/// - `Mapping`: overlay keys win; keys present only in `base` are preserved.
/// - `Document` / `Documents`: merge the root contents recursively.
/// - Everything else: `overlay` value wins outright.
fn deep_merge(base: &Node, overlay: &Node) -> Node {
    match (base, overlay) {
        // Merge two multi-document streams by merging the first document of each
        (Node::Documents(bd), Node::Documents(od)) => {
            match (bd.first(), od.first()) {
                (Some(b), Some(o)) => Node::Documents(vec![deep_merge(b, o)]),
                _ => overlay.clone(),
            }
        }

        // Merge two document bodies
        (Node::Document(bi), Node::Document(oi)) => {
            match (bi.first(), oi.first()) {
                (Some(b), Some(o)) => Node::Document(vec![deep_merge(b, o)]),
                _ => overlay.clone(),
            }
        }

        // Core case: merge two mappings
        (Node::Mapping(base_pairs), Node::Mapping(over_pairs)) => {
            let mut merged = base_pairs.clone();

            for (ok, ov) in over_pairs {
                match merged.iter_mut().find(|(bk, _)| bk == ok) {
                    Some((_, bv)) => *bv = deep_merge(bv, ov), // recurse on sub-trees
                    None => merged.push((ok.clone(), ov.clone())), // new key from overlay
                }
            }

            Node::Mapping(merged)
        }

        // Scalar, array, set, etc. — overlay wins
        _ => overlay.clone(),
    }
}

// ---------------------------------------------------------------------------
// Helpers for navigating the merged config
// ---------------------------------------------------------------------------

/// Navigate `Documents → Document → root mapping → section → key → &str`.
fn get_str<'a>(cfg: &'a Node, section: &str, key: &str) -> &'a str {
    navigate(cfg, section, key)
        .and_then(|v| v.as_str())
        .unwrap_or("?")
}

/// Navigate `Documents → Document → root mapping → section → key → i32`.
fn get_i32(cfg: &Node, section: &str, key: &str) -> i32 {
    navigate(cfg, section, key)
        .and_then(|v| v.as_i32())
        .unwrap_or(-1)
}

/// Navigate `Documents → Document → root mapping → section → key`.
fn navigate<'a>(cfg: &'a Node, section: &str, key: &str) -> Option<&'a Node> {
    // Documents → first Document
    let doc = match cfg {
        Node::Documents(docs) => docs.first()?,
        other => other,
    };
    // Document → first content node (the root mapping)
    let root = match doc {
        Node::Document(items) => items.first()?,
        other => other,
    };
    root.get_key(section)?.get_key(key)
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

fn print_yaml(label: &str, node: &Node) {
    let mut dest = BufferDestination::new();
    stringify(node, &mut dest).expect("stringify failed");
    println!("--- {} ---", label);
    println!("{}", dest.to_string().trim_end());
    println!();
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== YAML Config Layers Example ===\n");

    // Parse each layer independently
    let base = parse_string(BASE).expect("parse BASE");
    let prod = parse_string(PROD).expect("parse PROD");
    let dev = parse_string(DEV).expect("parse DEV");
    let local = parse_string(LOCAL).expect("parse LOCAL");

    // Print the raw layers so we can see what each contributes
    print_yaml("Layer 1 – base (defaults)", &base);
    print_yaml("Layer 2 – prod overlay", &prod);
    print_yaml("Layer 2 – dev overlay", &dev);
    print_yaml("Layer 3 – local override", &local);

    // ------------------------------------------------------------------
    // Merge: Production = base + prod
    // ------------------------------------------------------------------
    let production = deep_merge(&base, &prod);
    print_yaml("Effective production config", &production);

    // ------------------------------------------------------------------
    // Merge: Development = base + dev + local
    // ------------------------------------------------------------------
    let development = deep_merge(&base, &dev);
    let development = deep_merge(&development, &local);
    print_yaml("Effective development config", &development);

    // ------------------------------------------------------------------
    // Summary table: compare key values across environments
    // ------------------------------------------------------------------
    println!("--- Config comparison table ---");
    println!(
        "  {:<12} {:<8} {:<10} {:<10} {:<10} {:<8}",
        "env", "port", "log_level", "db_host", "pool", "cache"
    );
    println!("  {}", "-".repeat(60));

    let configs = [
        ("base", &base),
        ("production", &production),
        ("development", &development),
    ];

    for (name, cfg) in &configs {
        let port = get_i32(cfg, "app", "port");
        let log_level = get_str(cfg, "app", "log_level");
        let db_host = get_str(cfg, "database", "host");
        let pool = get_i32(cfg, "database", "pool_size");
        let cache = get_str(cfg, "cache", "enabled");

        println!(
            "  {:<12} {:<8} {:<10} {:<10} {:<10} {:<8}",
            name, port, log_level, db_host, pool, cache
        );
    }

    println!();

    // ------------------------------------------------------------------
    // Demonstrate: merging three or more layers generically
    // ------------------------------------------------------------------
    println!("--- Generic N-layer merge ---");
    let layers: &[(&str, &Node)] = &[
        ("base", &base),
        ("dev", &dev),
        ("local", &local),
    ];

    let effective = layers
        .iter()
        .skip(1)
        .fold(layers[0].1.clone(), |acc, (name, layer)| {
            println!("  applying {} overlay …", name);
            deep_merge(&acc, layer)
        });

    println!();
    print_yaml("Final effective config (N-layer fold)", &effective);
}
