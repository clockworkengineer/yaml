use crate::io::destinations::buffer::Buffer as BufferDestination;
use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::default::stringify as yaml_stringify;

// Reuse JSON-style escaping for TOML basic strings
fn escape_toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn write_toml_string(s: &str, destination: &mut dyn IDestination) {
    destination.add_byte(b'"');
    destination.add_bytes(&escape_toml_string(s));
    destination.add_byte(b'"');
}

fn node_to_key_string(key: &Node) -> Result<String, String> {
    match key {
        Node::Str(s, _, _) => Ok(s.clone()),
        Node::Number(num) => match num {
            Numeric::Integer(i) => Ok(i.to_string()),
            Numeric::Float(f) => Ok(f.to_string()),
            Numeric::UInteger(u) => Ok(u.to_string()),
            Numeric::Byte(b) => Ok(b.to_string()),
            Numeric::Int32(i) => Ok(i.to_string()),
            Numeric::UInt32(u) => Ok(u.to_string()),
            Numeric::Int16(i) => Ok(i.to_string()),
            Numeric::UInt16(u) => Ok(u.to_string()),
            Numeric::Int8(i) => Ok(i.to_string()),
        },
        Node::Boolean(b) => Ok((if *b { "true" } else { "false" }).to_string()),
        Node::None => Ok("".to_string()),
        _ => {
            // Fall back to YAML stringify for complex keys
            let mut buf = BufferDestination::new();
            yaml_stringify(key, &mut buf).map_err(|e| e)?;
            Ok(buf.to_string())
        }
    }
}

// (removed unused flatten_mapping helper)

fn is_array_of_maps(node: &Node) -> bool {
    match node {
        Node::Array(items) => items.iter().all(|it| matches!(it, Node::Mapping(_))),
        _ => false,
    }
}

// Write scalar (non-mapping) key/value pairs from `pairs`. If `write_newline` is true,
// prepend a newline before writing the first entry.
fn write_scalar_entries(
    pairs: &Vec<(Node, Node)>,
    prefix: Option<&str>,
    destination: &mut dyn IDestination,
    write_newline: &mut bool,
) -> Result<(), String> {
    for (k, v) in pairs.iter() {
        // skip mapping values and arrays of mappings here
        if matches!(v, Node::Mapping(_)) || is_array_of_maps(v) {
            continue;
        }
        // skip None values
        if let Node::None = v {
            continue;
        }
        if *write_newline {
            destination.add_byte(b'\n');
        }
        *write_newline = true;

        // When writing scalar entries inside a table, we emit the local key only.
        // For top-level we also emit the local key (no dotted prefix) because
        // nested tables are represented by headers.
        let key_str = node_to_key_string(k)?;
        destination.add_bytes(&key_str);
        destination.add_bytes(" = ");
        write_toml_value(v, destination)?;
    }
    // prefix is kept for potential future use; silence unused variable warning explicitly
    let _ = prefix;
    Ok(())
}

// Write a table recursively. If prefix is None, writes top-level scalar entries first.
fn write_table(
    pairs: &Vec<(Node, Node)>,
    prefix: Option<&str>,
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    // First write scalar entries at this level
    let mut wrote_any = false;
    write_scalar_entries(pairs, prefix, destination, &mut wrote_any)?;

    // Handle arrays-of-maps (array of tables)
    for (k, v) in pairs.iter() {
        if is_array_of_maps(v) {
            let key_str = node_to_key_string(k)?;
            let full_key = if let Some(p) = prefix {
                if p.is_empty() {
                    key_str.clone()
                } else if key_str.is_empty() {
                    p.to_string()
                } else {
                    format!("{}.{}", p, key_str)
                }
            } else {
                key_str.clone()
            };

            if let Node::Array(items) = v {
                for item in items.iter() {
                    if let Node::Mapping(inner) = item {
                        // array-of-tables header
                        if wrote_any {
                            destination.add_byte(b'\n');
                        }
                        wrote_any = true;
                        destination.add_bytes("[[");
                        destination.add_bytes(&full_key);
                        destination.add_bytes("]]");
                        // write inner scalar entries for this table
                        write_scalar_entries(inner, Some(&full_key), destination, &mut true)?;
                        // recurse for nested maps within this element
                        for (ik, iv) in inner.iter() {
                            if matches!(iv, Node::Mapping(_)) || is_array_of_maps(iv) {
                                // recurse with element's key as prefix
                                if let Node::Mapping(_) = iv {
                                    write_table(
                                        &vec![(ik.clone(), iv.clone())],
                                        Some(&full_key),
                                        destination,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Then handle nested mappings (as tables)
    for (k, v) in pairs.iter() {
        if let Node::Mapping(inner) = v {
            let key_str = node_to_key_string(k)?;
            let full_key = if let Some(p) = prefix {
                if p.is_empty() {
                    key_str.clone()
                } else if key_str.is_empty() {
                    p.to_string()
                } else {
                    format!("{}.{}", p, key_str)
                }
            } else {
                key_str.clone()
            };

            // header
            if wrote_any {
                destination.add_byte(b'\n');
            }
            wrote_any = true;
            destination.add_bytes("[");
            destination.add_bytes(&full_key);
            destination.add_bytes("]");

            // write scalar entries inside the table
            write_scalar_entries(inner, Some(&full_key), destination, &mut true)?;

            // recurse to write deeper nested tables
            for (ik, iv) in inner.iter() {
                if matches!(iv, Node::Mapping(_)) || is_array_of_maps(iv) {
                    if let Node::Mapping(_) = iv {
                        write_table(
                            &vec![(ik.clone(), iv.clone())],
                            Some(&full_key),
                            destination,
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn write_toml_value(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => {
            // TOML doesn't support null; caller should omit keys with None.
            // If we ever get a standalone None, emit an empty string.
            destination.add_bytes("\"\"")
        }
        Node::Boolean(b) => destination.add_bytes(if *b { "true" } else { "false" }),
        Node::Str(s, _, _) => write_toml_string(s, destination),
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&i.to_string()),
            Numeric::Float(f) => destination.add_bytes(&f.to_string()),
            Numeric::UInteger(u) => destination.add_bytes(&u.to_string()),
            Numeric::Byte(b) => destination.add_bytes(&b.to_string()),
            Numeric::Int32(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt32(u) => destination.add_bytes(&u.to_string()),
            Numeric::Int16(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt16(u) => destination.add_bytes(&u.to_string()),
            Numeric::Int8(i) => destination.add_bytes(&i.to_string()),
        },
        Node::Array(items) => {
            destination.add_byte(b'[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    destination.add_bytes(", ");
                }
                write_toml_value(it, destination)?;
            }
            destination.add_byte(b']');
        }
        Node::Mapping(pairs) => {
            // For mappings we prefer to output TOML tables for nested mappings
            // and array-of-tables for arrays of mappings. Use a dedicated
            // table writer to keep output idiomatic.
            write_table(pairs, None, destination)?;
        }
        Node::Document(nodes) => {
            if nodes.len() == 1 {
                write_toml_value(&nodes[0], destination)?;
            } else {
                // multiple top-level nodes: write each separated by a newline
                for (i, n) in nodes.iter().enumerate() {
                    if i > 0 {
                        destination.add_byte(b'\n');
                    }
                    write_toml_value(n, destination)?;
                }
            }
        }
        Node::Tagged(inner, _tag) => write_toml_value(inner, destination)?,
        Node::Anchored(inner, _name) => write_toml_value(inner, destination)?,
        Node::Alias(_name) => destination.add_bytes("\"\""),
        Node::Documents(docs) => {
            for (i, d) in docs.iter().enumerate() {
                if i > 0 {
                    destination.add_byte(b'\n');
                }
                write_toml_value(d, destination)?;
            }
        }
        Node::Comment(_) => {
            // Emit TOML comment
            // Skip comment content for now to keep output deterministic
        }
    }
    Ok(())
}

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Mapping(pairs) => write_table(pairs, None, destination),
        other => write_toml_value(other, destination),
    }
}

/// Pretty variant currently delegates to the same behaviour (TOML key/value layout is line-oriented).
pub fn stringify_pretty(
    node: &Node,
    destination: &mut dyn IDestination,
    _spaces_per_indent: usize,
) -> Result<(), String> {
    // For now pretty is same as non-pretty; TOML output here is already human readable
    stringify(node, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer as BufferDestination;
    use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};

    #[test]
    fn test_toml_basic() {
        let mut buf = BufferDestination::new();
        let n = Node::Str(
            "hello\nworld".to_string(),
            QuoteType::Unquoted,
            BlockStyle::None,
        );
        stringify(&n, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "\"hello\\nworld\"");

        buf.clear();
        let ni = Node::Number(Numeric::Integer(10));
        stringify(&ni, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "10");

        // mapping
        buf.clear();
        let m = Node::Mapping(vec![(
            Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(1),
        )]);
        stringify(&m, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "a = 1");

        // nested mapping -> table
        buf.clear();
        let nested = Node::Mapping(vec![(
            Node::Str("parent".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![(
                Node::Str("child".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::from(2),
            )]),
        )]);
        stringify(&nested, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "[parent]\nchild = 2");
    }
}
