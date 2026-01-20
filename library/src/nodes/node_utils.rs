//! Centralized helpers for node normalization, deduplication, and type-checking

use super::node::Node;

/// Normalize a node (e.g., convert empty strings to None, trim whitespace, etc.)
pub fn normalize_node(node: &Node) -> Node {
    // Example normalization logic (customize as needed)
    match node {
        Node::Str(s, q, b) if s.trim().is_empty() => Node::None,
        Node::Str(s, q, b) => Node::Str(s.trim().to_string(), q.clone(), b.clone()),
        _ => node.clone(),
    }
}

/// Deduplicate nodes in an array or set
pub fn deduplicate_nodes(nodes: &[Node]) -> Vec<Node> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for node in nodes {
        let key = format!("{:?}", node);
        if seen.insert(key) {
            result.push(node.clone());
        }
    }
    result
}

/// Check node type
pub fn is_string_node(node: &Node) -> bool {
    matches!(node, Node::Str(_, _, _))
}

pub fn is_number_node(node: &Node) -> bool {
    matches!(node, Node::Number(_))
}

pub fn is_boolean_node(node: &Node) -> bool {
    matches!(node, Node::Boolean(_))
}

pub fn is_none_node(node: &Node) -> bool {
    matches!(node, Node::None)
}
