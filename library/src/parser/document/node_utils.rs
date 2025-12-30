/// Construct a mapping node from key-value pairs.
pub fn make_mapping_node(pairs: Vec<(Node, Node)>) -> Node {
    Node::Mapping(pairs)
}

/// Construct an array node from items.
pub fn make_array_node(items: Vec<Node>) -> Node {
    Node::Array(items)
}

/// Construct a block scalar node (literal | or folded >)
pub fn make_block_scalar_node(content: String, is_folded: bool) -> Node {
    let style = if is_folded {
        BlockStyle::Folded
    } else {
        BlockStyle::Literal
    };
    Node::Str(content, QuoteType::Unquoted, style)
}
/// Shared node construction helpers for parser modules

use crate::nodes::node::{BlockStyle, Node, QuoteType};
use crate::parser::document::helpers::node_to_inline_string;

/// Normalize a Node to a double-quoted Node::Str for use as a mapping key.
pub fn normalize_node_to_str(node: &Node) -> Node {
    match node {
        Node::Array(_) | Node::Mapping(_) => {
            let inline = node_to_inline_string(node);
            Node::Str(inline, QuoteType::Double, BlockStyle::None)
        }
        Node::Str(s, _qt, style) => {
            let key_string = if matches!(style, BlockStyle::Literal) {
                format!("{}\n", s)
            } else {
                s.clone()
            };
            Node::Str(key_string, QuoteType::Double, BlockStyle::None)
        }
        other => {
            let inline = node_to_inline_string(other);
            Node::Str(inline, QuoteType::Double, BlockStyle::None)
        }
    }
}

/// Forcibly convert any mapping key to a string node (double quoted)
pub fn force_key_to_string(key: Node) -> Node {
    match key {
        Node::Str(_, _, _) => key,
        Node::Array(items) => {
            let mut s = String::from("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                match item {
                    Node::Str(val, _, _) => s.push_str(val),
                    Node::Number(n) => s.push_str(&format!("{:?}", n)),
                    Node::Boolean(b) => s.push_str(&format!("{}", b)),
                    _ => s.push_str(&format!("{:?}", item)),
                }
            }
            s.push(']');
            Node::Str(s, QuoteType::Double, BlockStyle::None)
        }
        Node::Mapping(_) => {
            Node::Str(format!("{:?}", key), QuoteType::Double, BlockStyle::None)
        }
        Node::Tagged(inner, tag) => {
            if matches!(*inner, Node::Str(ref s, _, _) if s.is_empty()) {
                Node::Tagged(inner, tag)
            } else {
                Node::Tagged(Box::new(force_key_to_string(*inner)), tag)
            }
        }
        Node::Anchored(inner, name) => {
            if matches!(*inner, Node::Str(ref s, _, _) if s.is_empty()) {
                Node::Anchored(inner, name)
            } else {
                Node::Anchored(Box::new(force_key_to_string(*inner)), name)
            }
        }
        other => {
            Node::Str(format!("{:?}", other), QuoteType::Double, BlockStyle::None)
        }
    }
}
