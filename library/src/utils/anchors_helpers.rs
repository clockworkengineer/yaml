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
