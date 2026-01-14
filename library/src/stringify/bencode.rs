//! Module: stringify/bencode.rs

use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::format::node_to_key_like_string;

/// Recursively encodes a YAML node to Bencode format.
///
/// Converts YAML nodes to their Bencode equivalents, handling strings,
/// integers, lists (arrays), and dictionaries (mappings) according to
/// the Bencode specification.
///
/// # Arguments
///
/// * `node` - The Node to encode
/// * `destination` - The output destination for the Bencode data
///
/// # Returns
///
/// Result indicating success or an error string
fn encode_node(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => {
            destination.add_bytes("0:");
            Ok(())
        }
        Node::Comment(c) => {
            destination.add_bytes(&format!("{}:", c.len()));
            destination.add_bytes(c);
            Ok(())
        }
        Node::Boolean(b) => {
            let s = if *b { "true" } else { "false" };
            destination.add_bytes(&format!("{}:", s.len()));
            destination.add_bytes(s);
            Ok(())
        }
        Node::Str(s, _qt, _style) => {
            destination.add_bytes(&format!("{}:", s.as_bytes().len()));
            destination.add_bytes(s);
            Ok(())
        }

        Node::Number(num) => match num {
            Numeric::Integer(i) => {
                destination.add_bytes(&format!("i{}e", i));
                Ok(())
            }
            Numeric::Float(f) => {
                let s = f.to_string();
                destination.add_bytes(&format!("{}:", s.as_bytes().len()));
                destination.add_bytes(&s);
                Ok(())
            }

            Numeric::UInteger(u) => {
                destination.add_bytes(&format!("i{}e", u));
                Ok(())
            }
            Numeric::Byte(b) => {
                destination.add_bytes(&format!("i{}e", b));
                Ok(())
            }
            Numeric::Int32(i) => {
                destination.add_bytes(&format!("i{}e", i));
                Ok(())
            }
            Numeric::UInt32(u) => {
                destination.add_bytes(&format!("i{}e", u));
                Ok(())
            }
            Numeric::Int16(i) => {
                destination.add_bytes(&format!("i{}e", i));
                Ok(())
            }
            Numeric::UInt16(u) => {
                destination.add_bytes(&format!("i{}e", u));
                Ok(())
            }
            Numeric::Int8(i) => {
                destination.add_bytes(&format!("i{}e", i));
                Ok(())
            }
            Numeric::UInt8(u) => {
                destination.add_bytes(&format!("i{}e", u));
                Ok(())
            }
        },
        Node::Array(items) => {
            destination.add_bytes("l");
            for item in items {
                encode_node(item, destination)?;
            }
            destination.add_bytes("e");
            Ok(())
        }
        Node::Set(items) => {
            // Represent sets as bencode lists in bencode output
            destination.add_bytes("l");
            for item in items {
                encode_node(item, destination)?;
            }
            destination.add_bytes("e");
            Ok(())
        }
        Node::Mapping(pairs) => {
            let mut entries: Vec<(Vec<u8>, &Node)> = Vec::new();
            for (k, v) in pairs.iter() {
                let key_str = node_to_key_like_string(k);
                let key_bytes: Vec<u8> = key_str.into_bytes();
                entries.push((key_bytes, v));
            }

            entries.sort_by(|a, b| a.0.cmp(&b.0));

            destination.add_bytes("d");
            for (key_bytes, value_node) in entries.iter() {
                destination.add_bytes(&format!("{}:", key_bytes.len()));
                for &b in key_bytes.iter() {
                    destination.add_byte(b);
                }
                encode_node(value_node, destination)?;
            }
            destination.add_bytes("e");
            Ok(())
        }
        Node::Document(nodes) => {
            if nodes.len() == 1 {
                encode_node(&nodes[0], destination)
            } else {
                destination.add_bytes("l");
                for n in nodes {
                    encode_node(n, destination)?;
                }
                destination.add_bytes("e");
                Ok(())
            }
        }
        Node::Tagged(inner, _tag) => encode_node(inner, destination),
        Node::Anchored(inner, _name) => encode_node(inner, destination),
        Node::Alias(_name) => {
            destination.add_bytes("0:");
            Ok(())
        }
        Node::Documents(docs) => {
            destination.add_bytes("l");
            for d in docs {
                encode_node(d, destination)?;
            }
            destination.add_bytes("e");
            Ok(())
        }
    }
}

/// Converts YAML nodes to Bencode format.
///
/// Main entry point for Bencode stringification. Converts YAML document
/// structures to valid Bencode encoding, commonly used in BitTorrent protocols.
///
/// # Arguments
///
/// * `node` - The root Node to convert to Bencode
/// * `destination` - The output destination for the Bencode data
///
/// # Returns
///
/// Result indicating success or an error string
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    encode_node(node, destination)
}
