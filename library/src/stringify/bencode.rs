//! Bencode Stringification
//!
//! Provides functions for recursively encoding YAML nodes to Bencode format, handling strings,
//! integers, lists, and dictionaries according to the Bencode specification.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::format::node_to_key_like_string;
use crate::stringify::traits::NodeSerializer;

/// Bencode Serializer implementing `NodeSerializer` (OCP & DIP)
#[derive(Debug, Default, Clone, Copy)]
pub struct BencodeSerializer;

impl NodeSerializer for BencodeSerializer {
    fn serialize(&self, node: &Node, dest: &mut dyn IDestination) -> crate::error::Result<()> {
        stringify(node, dest).map_err(Into::into)
    }
}

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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::traits::IDestination;
    use crate::nodes::node::{Node, Numeric};
    use std::cell::RefCell;

    struct TestDestination {
        pub data: RefCell<Vec<u8>>,
    }

    impl TestDestination {
        fn new() -> Self {
            Self {
                data: RefCell::new(Vec::new()),
            }
        }
        fn as_str(&self) -> String {
            String::from_utf8(self.data.borrow().clone()).unwrap()
        }
    }

    impl IDestination for TestDestination {
        fn add_bytes(&mut self, bytes: &str) {
            self.data.borrow_mut().extend_from_slice(bytes.as_bytes());
        }
        fn add_byte(&mut self, byte: u8) {
            self.data.borrow_mut().push(byte);
        }
        fn clear(&mut self) {
            self.data.borrow_mut().clear();
        }
        fn last(&self) -> Option<u8> {
            self.data.borrow().last().copied()
        }
    }

    #[test]
    fn test_stringify_none() {
        let node = Node::None;
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "0:");
    }

    #[test]
    fn test_stringify_boolean() {
        let node_true = Node::Boolean(true);
        let node_false = Node::Boolean(false);
        let mut dest_true = TestDestination::new();
        let mut dest_false = TestDestination::new();
        stringify(&node_true, &mut dest_true).unwrap();
        stringify(&node_false, &mut dest_false).unwrap();
        assert_eq!(dest_true.as_str(), "4:true");
        assert_eq!(dest_false.as_str(), "5:false");
    }

    #[test]
    fn test_stringify_str() {
        let node = Node::Str("hello".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "5:hello");
    }

    #[test]
    fn test_stringify_integer() {
        let node = Node::Number(Numeric::Integer(42));
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "i42e");
    }

    #[test]
    fn test_stringify_float() {
        let node = Node::Number(Numeric::Float(3.14));
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "4:3.14");
    }

    #[test]
    fn test_stringify_array() {
        let node = Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Str("foo".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ]);
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "li1e3:fooe");
    }

    #[test]
    fn test_stringify_mapping() {
        let node = Node::Mapping(vec![
            (
                Node::Str("bar".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(2)),
            ),
            (
                Node::Str("foo".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(1)),
            ),
        ]);
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        // Keys are sorted lexicographically: bar, foo
        assert_eq!(dest.as_str(), "d3:bari2e3:fooi1ee");
    }

    #[test]
    fn test_stringify_document() {
        let node = Node::Document(vec![Node::Str(
            "doc".to_string(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )]);
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "3:doc");
    }

    #[test]
    fn test_stringify_documents() {
        let node = Node::Documents(vec![
            Node::Str("doc1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("doc2".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ]);
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "l4:doc14:doc2e");
    }

    #[test]
    fn test_stringify_tagged_and_anchored() {
        let tagged = Node::Tagged(
            Box::new(Node::Str(
                "tagged".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            String::new(),
        );
        let anchored = Node::Anchored(
            Box::new(Node::Str(
                "anchored".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            String::new(),
        );
        let mut dest_tagged = TestDestination::new();
        let mut dest_anchored = TestDestination::new();
        stringify(&tagged, &mut dest_tagged).unwrap();
        stringify(&anchored, &mut dest_anchored).unwrap();
        assert_eq!(dest_tagged.as_str(), "6:tagged");
        assert_eq!(dest_anchored.as_str(), "8:anchored");
    }

    #[test]
    fn test_stringify_alias() {
        let node = Node::Alias("alias_name".to_string());
        let mut dest = TestDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.as_str(), "0:");
    }
}
