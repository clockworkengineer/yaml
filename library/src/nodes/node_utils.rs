/// Check if node is a string node
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

pub fn is_number_node(node: &Node) -> bool {
    matches!(node, Node::Number(_))
}

pub fn is_boolean_node(node: &Node) -> bool {
    matches!(node, Node::Boolean(_))
}

pub fn is_none_node(node: &Node) -> bool {
    matches!(node, Node::None)
}
