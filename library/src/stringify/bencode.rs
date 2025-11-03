use crate::io::destinations::buffer::Buffer as BufferDestination;
use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::default::stringify as yaml_stringify;

// Encode a node into bencode format and write to destination.
fn encode_node(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => {
            // Represent null as empty string
            destination.add_bytes("0:");
            Ok(())
        }
        Node::Comment(c) => {
            // Encode comments as strings (preserve content)
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
        // Note: the Node::Binary variant was removed from node.rs in a recent edit.
        // Binary data should be represented as strings in this build; fall back to
        // treating any binary-like data as a string if present as such.
        Node::Number(num) => {
            match num {
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
                // Other integer-like numeric types: encode as integers
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
            }
        }
        Node::Array(items) => {
            destination.add_bytes("l");
            for item in items {
                encode_node(item, destination)?;
            }
            destination.add_bytes("e");
            Ok(())
        }
        Node::Mapping(pairs) => {
            // Convert keys to raw bytes and sort lexicographically per bencode spec
            let mut entries: Vec<(Vec<u8>, &Node)> = Vec::new();
            for (k, v) in pairs.iter() {
                // produce bytes for key
                let key_bytes: Vec<u8> = match k {
                    Node::Str(s, _, _) => s.as_bytes().to_vec(),
                    Node::Number(n) => match n {
                        Numeric::Integer(i) => i.to_string().as_bytes().to_vec(),
                        Numeric::Float(f) => f.to_string().as_bytes().to_vec(),
                        Numeric::UInteger(u) => u.to_string().as_bytes().to_vec(),
                        Numeric::Byte(bv) => bv.to_string().as_bytes().to_vec(),
                        Numeric::Int32(i) => i.to_string().as_bytes().to_vec(),
                        Numeric::UInt32(u) => u.to_string().as_bytes().to_vec(),
                        Numeric::Int16(i) => i.to_string().as_bytes().to_vec(),
                        Numeric::UInt16(u) => u.to_string().as_bytes().to_vec(),
                        Numeric::Int8(i) => i.to_string().as_bytes().to_vec(),
                    },
                    Node::Boolean(bv) => {
                        let s = if *bv { "true" } else { "false" };
                        s.as_bytes().to_vec()
                    }
                    Node::None => Vec::new(),
                    _ => {
                        // fall back to YAML stringify for complex keys
                        let mut buf = BufferDestination::new();
                        yaml_stringify(k, &mut buf)?;
                        buf.to_string().into_bytes()
                    }
                };
                entries.push((key_bytes, v));
            }
            // sort lexicographically by raw byte sequence
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
                // represent multi-node document as list
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
            // No anchor resolution here; encode as empty string
            destination.add_bytes("0:");
            Ok(())
        }
        Node::Documents(docs) => {
            // Represent a Documents collection as a list of documents
            destination.add_bytes("l");
            for d in docs {
                encode_node(d, destination)?;
            }
            destination.add_bytes("e");
            Ok(())
        }
    }
}

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    encode_node(node, destination)
}
