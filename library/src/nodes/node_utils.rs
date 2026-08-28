
//! Node Utility Functions
//!
//! Provides helper functions for constructing tagged, anchored, and mapping nodes
//! in the YAML library. Used for building and manipulating node structures programmatically.
//!
//! Copyright (c) 2026 YAML Library Developers

use super::node::{BlockStyle, QuoteType};

#[allow(dead_code)]
/// Construct a tagged node from an inner node and tag string.
pub fn make_tagged_node(inner: Node, tag: String) -> Node {
    Node::Tagged(Box::new(inner), tag)
}

#[allow(dead_code)]
/// Construct an anchored node from an inner node and anchor name.
pub fn make_anchored_node(inner: Node, name: String) -> Node {
    Node::Anchored(Box::new(inner), name)
}

/// Construct a mapping node from key-value pairs.
pub fn make_mapping_node(pairs: Vec<(Node, Node)>) -> Node {
    Node::Mapping(pairs)
}

#[allow(dead_code)]
/// Construct an array node from items.
pub fn make_array_node(items: Vec<Node>) -> Node {
    Node::Array(items)
}

#[allow(dead_code)]
/// Construct a block scalar node (literal | or folded >)
pub fn make_block_scalar_node(content: String, is_folded: bool) -> Node {
    let style = if is_folded {
        BlockStyle::Folded
    } else {
        BlockStyle::Literal
    };
    Node::Str(content, QuoteType::Unquoted, style)
}
/// Check if node is a string node
#[inline]
pub fn is_string_node(node: &Node) -> bool {
    matches!(node, Node::Str(_, _, _))
}

/// Construct a set node from items
pub fn make_set_node(items: Vec<Node>) -> Node {
    Node::Set(items)
}
/// Centralized helpers for node normalization, deduplication, and type-checking
use super::node::Node;

/// Normalize a node (e.g., convert empty strings to None, trim whitespace, etc.)
pub fn normalize_node(node: &Node) -> Node {
    // Example normalization logic (customize as needed)
    match node {
        Node::Str(s, ..) if s.trim().is_empty() => Node::None,
        Node::Str(s, q, b) => Node::Str(s.trim().to_string(), q.clone(), b.clone()),
        _ => node.clone(),
    }
}

/// Deduplicate nodes in an array or set

#[inline]
pub fn is_number_node(node: &Node) -> bool {
    matches!(node, Node::Number(_))
}

#[inline]
pub fn is_boolean_node(node: &Node) -> bool {
    matches!(node, Node::Boolean(_))
}

#[inline]
pub fn is_none_node(node: &Node) -> bool {
    matches!(node, Node::None)
}

/// Returns the current version of the package as specified in Cargo.toml.
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the number of documents in a YAML stream represented by the Documents node.
pub fn get_number_of_documents(documents: &Node) -> Result<usize, String> {
    match documents {
        Node::Documents(docs) => Ok(docs.len()),
        _ => Err("Expected Documents node".to_string()),
    }
}

/// Returns the base node of document number n (0-based), reporting any errors.
pub fn get_document_base(node: &Node, n: usize) -> Result<&Node, String> {
    match node {
        Node::Documents(docs) => {
            if n < docs.len() {
                Ok(&docs[n])
            } else {
                Err(format!(
                    "Document index {} out of bounds ({} documents)",
                    n,
                    docs.len()
                ))
            }
        }
        _ => Err("Expected Documents node".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::{Node, QuoteType, BlockStyle};

    #[test]
    fn test_make_tagged_node() {
        let inner = Node::from("foo");
        let tagged = make_tagged_node(inner.clone(), "!mytag".to_string());
        match tagged {
            Node::Tagged(boxed, tag) => {
                assert_eq!(*boxed, inner);
                assert_eq!(tag, "!mytag");
            }
            _ => panic!("Expected Tagged node"),
        }
    }

    #[test]
    fn test_make_anchored_node() {
        let inner = Node::from(42);
        let anchored = make_anchored_node(inner.clone(), "anchor1".to_string());
        match anchored {
            Node::Anchored(boxed, name) => {
                assert_eq!(*boxed, inner);
                assert_eq!(name, "anchor1");
            }
            _ => panic!("Expected Anchored node"),
        }
    }

    #[test]
    fn test_make_mapping_node() {
        let pairs = vec![(Node::from("k"), Node::from("v"))];
        let mapping = make_mapping_node(pairs.clone());
        match mapping {
            Node::Mapping(m) => assert_eq!(m, pairs),
            _ => panic!("Expected Mapping node"),
        }
    }

    #[test]
    fn test_make_array_node() {
        let items = vec![Node::from(1), Node::from(2)];
        let arr = make_array_node(items.clone());
        match arr {
            Node::Array(a) => assert_eq!(a, items),
            _ => panic!("Expected Array node"),
        }
    }

    #[test]
    fn test_make_block_scalar_node_literal_and_folded() {
        let lit = make_block_scalar_node("abc".to_string(), false);
        let fold = make_block_scalar_node("xyz".to_string(), true);
        match lit {
            Node::Str(s, QuoteType::Unquoted, BlockStyle::Literal) => assert_eq!(s, "abc"),
            _ => panic!("Expected literal block scalar"),
        }
        match fold {
            Node::Str(s, QuoteType::Unquoted, BlockStyle::Folded) => assert_eq!(s, "xyz"),
            _ => panic!("Expected folded block scalar"),
        }
    }

    #[test]
    fn test_is_string_node_and_is_number_node() {
        let s = Node::from("foo");
        let n = Node::from(123);
        assert!(is_string_node(&s));
        assert!(!is_string_node(&n));
        assert!(is_number_node(&n));
        assert!(!is_number_node(&s));
    }

    #[test]
    fn test_is_boolean_node_and_is_none_node() {
        let b = Node::from(true);
        let n = Node::None;
        assert!(is_boolean_node(&b));
        assert!(!is_boolean_node(&n));
        assert!(is_none_node(&n));
        assert!(!is_none_node(&b));
    }

    #[test]
    fn test_make_set_node() {
        let items = vec![Node::from(1), Node::from(2)];
        let set = make_set_node(items.clone());
        match set {
            Node::Set(s) => assert_eq!(s, items),
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_normalize_node() {
        let n1 = Node::from("");
        let n2 = Node::from("  ");
        let n3 = Node::from("abc");
        let n4 = Node::from(123);
        assert_eq!(normalize_node(&n1), Node::None);
        assert_eq!(normalize_node(&n2), Node::None);
        assert_eq!(normalize_node(&n3), Node::from("abc"));
        assert_eq!(normalize_node(&n4), n4);
    }
}

