//! Node Utility Functions
//!
//! Provides shared utility functions for node creation and manipulation in the YAML library.
//! Includes helpers for constructing nodes and sets from generic values.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::nodes::node::Node;

/// Create a Node from any value that can be converted into a Node
pub fn make_node<T: Into<Node>>(value: T) -> Node {
    value.into()
}

/// Create a Set Node from a vector of values, ensuring uniqueness
pub fn make_set<T: Into<Node> + Clone>(values: Vec<T>) -> Node {
    let mut unique_nodes = Vec::new();
    for value in values {
        let node = value.into();
        if !unique_nodes.contains(&node) {
            unique_nodes.push(node);
        }
    }
    Node::Set(unique_nodes)
}
