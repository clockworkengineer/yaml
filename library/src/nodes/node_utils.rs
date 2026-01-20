use super::node::{BlockStyle, QuoteType};

/// Construct a tagged node from an inner node and tag string.
#[allow(dead_code)]
pub fn make_tagged_node(inner: Node, tag: String) -> Node {
    Node::Tagged(Box::new(inner), tag)
}

/// Construct an anchored node from an inner node and anchor name.
#[allow(dead_code)]
pub fn make_anchored_node(inner: Node, name: String) -> Node {
    Node::Anchored(Box::new(inner), name)
}

/// Construct a mapping node from key-value pairs.
pub fn make_mapping_node(pairs: Vec<(Node, Node)>) -> Node {
    Node::Mapping(pairs)
}

/// Construct an array node from items.
#[allow(dead_code)]
pub fn make_array_node(items: Vec<Node>) -> Node {
    Node::Array(items)
}

/// Construct a block scalar node (literal | or folded >)
#[allow(dead_code)]
pub fn make_block_scalar_node(content: String, is_folded: bool) -> Node {
    let style = if is_folded {
        BlockStyle::Folded
    } else {
        BlockStyle::Literal
    };
    Node::Str(content, QuoteType::Unquoted, style)
}
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

#[allow(dead_code)]
pub fn is_none_node(node: &Node) -> bool {
    matches!(node, Node::None)
}
