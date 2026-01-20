/// Construct a set node from items.
pub mod node_utils {
}

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
/// Shared node construction helpers for parser modules
use crate::nodes::node::{BlockStyle, Node, QuoteType};
use crate::parser::document::helpers::node_to_inline_string;

/// Normalize a Node to a double-quoted Node::Str for use as a mapping key.
pub fn normalize_node_to_str(node: &Node) -> Node {
    use crate::nodes::node::{NodeStringConvert, QuoteType, BlockStyle};
    use crate::nodes::node_utils::{is_string_node, is_number_node, is_boolean_node, normalize_node};
    let normalized = normalize_node(node);
        match &normalized {
            n if is_string_node(n) => n.clone(),
            n if is_number_node(n) || is_boolean_node(n) => {
                let inline = n.to_string_lossy();
                Node::Str(inline, QuoteType::Double, BlockStyle::None)
            }
            Node::Array(items) => {
                let mut s = String::from("[");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    // Use as_str() if possible, else lossy
                    if let Some(str_val) = item.as_str() {
                        s.push_str(str_val);
                    } else {
                        s.push_str(&item.to_string_lossy());
                    }
                }
                s.push(']');
                Node::Str(s, QuoteType::Double, BlockStyle::None)
            }
            Node::Mapping(_) => {
                let inline = normalized.to_string_lossy();
                Node::Str(inline, QuoteType::Double, BlockStyle::None)
            }
            other => {
                let inline = other.to_string_lossy();
                Node::Str(inline, QuoteType::Double, BlockStyle::None)
            }
        }
}

/// Forcibly convert any mapping key to a string node (double quoted)
pub fn force_key_to_string(key: Node) -> Node {
    use crate::nodes::node::{NodeStringConvert, QuoteType, BlockStyle};
    use crate::nodes::node_utils::{is_string_node, is_number_node, is_boolean_node, normalize_node};
    let normalized = normalize_node(&key);
    if is_string_node(&normalized) {
        normalized
    } else if is_number_node(&normalized) || is_boolean_node(&normalized) {
        let inline = normalized.to_string_lossy();
        Node::Str(inline, QuoteType::Double, BlockStyle::None)
    } else if let Node::Array(items) = &normalized {
        let mut s = String::from("[");
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let inline = item.to_string_lossy();
            s.push_str(&inline);
        }
        s.push(']');
        Node::Str(s, QuoteType::Double, BlockStyle::None)
    } else if let Node::Mapping(_) = &normalized {
        let inline = normalized.to_string_lossy();
        Node::Str(inline, QuoteType::Double, BlockStyle::None)
    } else if let Node::Tagged(inner, tag) = &normalized {
        if matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()) {
            Node::Tagged(inner.clone(), tag.clone())
        } else {
            Node::Tagged(Box::new(force_key_to_string((**inner).clone())), tag.clone())
        }
    } else if let Node::Anchored(inner, name) = &normalized {
        if matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()) {
            Node::Anchored(inner.clone(), name.clone())
        } else {
            Node::Anchored(Box::new(force_key_to_string((**inner).clone())), name.clone())
        }
    } else {
        let inline = normalized.to_string_lossy();
        Node::Str(inline, QuoteType::Double, BlockStyle::None)
    }
}

/// Dedupe mapping by last key occurrence, preserving relative order of surviving pairs.
/// Keys are compared using inline string representation.
pub fn dedupe_mapping_pairs_by_last_occurrence(pairs: Vec<(Node, Node)>) -> Vec<(Node, Node)> {
    use std::collections::HashMap as Map;
    let mut last_index: Map<String, usize> = Map::new();
    for (idx, (k, _v)) in pairs.iter().enumerate() {
        let key_s = node_to_inline_string(k);
        last_index.insert(key_s, idx);
    }
    let mut rebuilt: Vec<(Node, Node)> = Vec::new();
    for (idx, (k, v)) in pairs.into_iter().enumerate() {
        let key_s = node_to_inline_string(&k);
        if let Some(&last) = last_index.get(&key_s) {
            if last == idx {
                rebuilt.push((k, v));
            }
        }
    }
    rebuilt
}

/// If all mapping values are Node::None, return the keys as set items; otherwise None.
pub fn pairs_to_set_items_if_all_none(pairs: &[(Node, Node)]) -> Option<Vec<Node>> {
    let mut items = Vec::new();
    for (k, v) in pairs.iter() {
        if matches!(v, Node::None) {
            items.push(k.clone());
        } else {
            return None;
        }
    }
    Some(items)
}

/// Tag helpers (for DRY checks on resolved tag strings)
/// The input should be a resolved tag string (e.g., "!!set" or "tag:yaml.org,2002:set").
#[inline]
pub fn resolved_is_set(tag: &str) -> bool {
    tag == "!!set" || tag == "tag:yaml.org,2002:set"
}

#[inline]
pub fn resolved_is_seq(tag: &str) -> bool {
    tag == "!!seq" || tag == "tag:yaml.org,2002:seq"
}

#[inline]
pub fn resolved_is_map(tag: &str) -> bool {
    tag == "!!map" || tag == "tag:yaml.org,2002:map"
}
