//! Anchor Helpers for YAML Processing
//!
//! This module provides helper functions for working with YAML anchors, including node traversal
//! with error propagation, anchor lookup, alias replacement, and merge key expansion. These utilities
//! support robust and DRY handling of YAML anchor and alias semantics.
//!
//! # Features
//! - Traverse node trees with error handling
//! - Lookup anchors by name
//! - Check and extract mapping nodes for merge operations
//!
//! # Usage
//! Use these helpers in parser and document modules to simplify anchor and alias logic.

use crate::nodes::node::Node;
use crate::parser::ParseResult;

/// Traverses a node tree, applying a closure to each node, propagating errors.
#[allow(dead_code)]
pub fn traverse_with_error<F>(node: &Node, mut f: F) -> ParseResult<()>
where
    F: FnMut(&Node) -> Option<String>,
{
    let mut err: Option<String> = None;
    let mut visit = |n: &Node| {
        if let Some(e) = f(n) {
            err = Some(e);
        }
    };
    crate::parser::utils::visit::visit(node, &mut visit);
    if let Some(e) = err {
        Err(crate::error::YamlError::from(e))
    } else {
        Ok(())
    }
}

// Helper functions for anchor lookup, alias replacement, and merge expansion
use std::collections::HashMap;

/// Looks up an anchor by name, returning a reference to the node or an error if not found.
#[allow(dead_code)]
pub fn lookup_anchor<'a>(
    anchors: &'a HashMap<String, Node>,
    name: &str,
) -> Result<&'a Node, crate::error::YamlError> {
    anchors
        .get(name)
        .ok_or_else(|| crate::parser::utils::error_helpers::undefined_anchor(name))
}

/// Checks if a node is a mapping and returns its pairs, or an error if not a mapping.
#[allow(dead_code)]
pub fn as_mapping<'a>(
    node: &'a Node,
    name: &str,
) -> Result<&'a Vec<(Node, Node)>, crate::error::YamlError> {
    if let Node::Mapping(pairs) = node {
        Ok(pairs)
    } else {
        Err(crate::parser::utils::error_helpers::merge_source_not_mapping(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::Node;
    use std::collections::HashMap;

    #[test]
    fn test_lookup_anchor_found() {
        let mut anchors = HashMap::new();
        let node = Node::None;
        anchors.insert("foo".to_string(), node.clone());
        let found = lookup_anchor(&anchors, "foo");
        assert!(found.is_ok());
        assert_eq!(found.unwrap(), &node);
    }

    #[test]
    fn test_lookup_anchor_not_found() {
        let anchors = HashMap::new();
        let err = lookup_anchor(&anchors, "bar").unwrap_err();
        assert!(err.message().contains("bar"));
    }

    #[test]
    fn test_as_mapping_success() {
        let pairs = vec![(Node::None, Node::None)];
        let node = Node::Mapping(pairs.clone());
        let result = as_mapping(&node, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &pairs);
    }

    #[test]
    fn test_as_mapping_error() {
        let node = Node::None;
        let result = as_mapping(&node, "notmap");
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("not a mapping"));
    }

    #[test]
    fn test_traverse_with_error_ok() {
        let node = Node::None;
        let res = traverse_with_error(&node, |_n| None);
        assert!(res.is_ok());
    }

    #[test]
    fn test_traverse_with_error_err() {
        let node = Node::None;
        let res = traverse_with_error(&node, |_n| Some("fail".to_string()));
        assert!(res.is_err());
        assert!(format!("{}", res.unwrap_err()).contains("fail"));
    }
}
