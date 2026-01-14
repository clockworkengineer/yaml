//! Node inspection and introspection tools
//!
//! Provides utilities for examining YAML node structure, types, and content.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::nodes::node::{Node, Numeric};
use crate::utils::streaming::NodeIteratorExt;

/// Node type information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Mapping,
    Set,
    Document,
    Documents,
    Tagged,
    Anchored,
    Alias,
    Comment,
}

impl NodeType {
    /// Get the type name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Null => "null",
            NodeType::Boolean => "boolean",
            NodeType::Integer => "integer",
            NodeType::Float => "float",
            NodeType::String => "string",
            NodeType::Array => "array",
            NodeType::Mapping => "mapping",
            NodeType::Set => "set",
            NodeType::Document => "document",
            NodeType::Documents => "documents",
            NodeType::Tagged => "tagged",
            NodeType::Anchored => "anchored",
            NodeType::Alias => "alias",
            NodeType::Comment => "comment",
        }
    }

    /// Check if this is a scalar type
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            NodeType::Null
                | NodeType::Boolean
                | NodeType::Integer
                | NodeType::Float
                | NodeType::String
        )
    }

    /// Check if this is a collection type
    pub fn is_collection(&self) -> bool {
        matches!(self, NodeType::Array | NodeType::Mapping | NodeType::Set)
    }
}

/// Get the type of a node
pub fn node_type(node: &Node) -> NodeType {
    match node {
        Node::None => NodeType::Null,
        Node::Boolean(_) => NodeType::Boolean,
        Node::Number(
            Numeric::Integer(_)
            | Numeric::UInteger(_)
            | Numeric::Int32(_)
            | Numeric::UInt32(_)
            | Numeric::Int16(_)
            | Numeric::UInt16(_)
            | Numeric::Int8(_)
            | Numeric::Byte(_),
        ) => NodeType::Integer,
        Node::Number(Numeric::Float(_)) => NodeType::Float,
        Node::Str(_, _, _) => NodeType::String,
        Node::Array(_) => NodeType::Array,
        Node::Mapping(_) => NodeType::Mapping,
        Node::Set(_) => NodeType::Set,
        Node::Document(_) => NodeType::Document,
        Node::Documents(_) => NodeType::Documents,
        Node::Tagged(_, _) => NodeType::Tagged,
        Node::Anchored(_, _) => NodeType::Anchored,
        Node::Alias(_) => NodeType::Alias,
        Node::Comment(_) => NodeType::Comment,
    }
}

/// Detailed node information
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub node_type: NodeType,
    pub size: usize,
    pub depth: usize,
    pub has_tag: bool,
    pub has_anchor: bool,
    pub is_alias: bool,
    pub summary: String,
}

impl NodeInfo {
    /// Create node information for a given node
    pub fn new(node: &Node) -> Self {
        Self {
            node_type: node_type(node),
            size: node_size(node),
            depth: node_depth(node),
            has_tag: has_tag(node),
            has_anchor: has_anchor(node),
            is_alias: matches!(node, Node::Alias(_)),
            summary: node_summary(node),
        }
    }

    /// Format as a readable string
    pub fn format(&self) -> String {
        format!(
            "Type: {}, Size: {}, Depth: {}, Tag: {}, Anchor: {}, Alias: {}\n{}",
            self.node_type.as_str(),
            self.size,
            self.depth,
            self.has_tag,
            self.has_anchor,
            self.is_alias,
            self.summary
        )
    }
}

/// Get the size (number of child nodes) of a node
pub fn node_size(node: &Node) -> usize {
    match node {
        Node::Array(items) => items.len(),
        Node::Mapping(pairs) => pairs.len(),
        Node::Set(items) => items.len(),
        Node::Document(items) => items.len(),
        Node::Documents(docs) => docs.len(),
        _ => 0,
    }
}

/// Get the maximum depth of a node tree
pub fn node_depth(node: &Node) -> usize {
    match node {
        Node::Array(items) => 1 + items.iter().map(node_depth).max().unwrap_or(0),
        Node::Mapping(pairs) => {
            1 + pairs
                .iter()
                .map(|(k, v)| node_depth(k).max(node_depth(v)))
                .max()
                .unwrap_or(0)
        }
        Node::Set(items) => 1 + items.iter().map(node_depth).max().unwrap_or(0),
        Node::Document(items) => 1 + items.iter().map(node_depth).max().unwrap_or(0),
        Node::Documents(docs) => 1 + docs.iter().map(node_depth).max().unwrap_or(0),
        Node::Tagged(inner, _) => 1 + node_depth(inner),
        Node::Anchored(inner, _) => 1 + node_depth(inner),
        _ => 1,
    }
}

/// Check if a node has a tag
pub fn has_tag(node: &Node) -> bool {
    matches!(node, Node::Tagged(_, _))
}

/// Check if a node has an anchor
pub fn has_anchor(node: &Node) -> bool {
    matches!(node, Node::Anchored(_, _))
}

/// Get a summary string for a node
pub fn node_summary(node: &Node) -> String {
    match node {
        Node::None => "null".to_string(),
        Node::Boolean(b) => format!("Boolean: {}", b),
        Node::Number(n) => format!("Number: {:?}", n),
        Node::Str(s, _, _) => {
            if s.len() > 50 {
                format!("String: \"{}...\" ({} chars)", &s[..47], s.len())
            } else {
                format!("String: \"{}\"", s)
            }
        }
        Node::Array(items) => format!("Array with {} items", items.len()),
        Node::Mapping(pairs) => format!("Mapping with {} pairs", pairs.len()),
        Node::Set(items) => format!("Set with {} items", items.len()),
        Node::Document(items) => format!("Document with {} nodes", items.len()),
        Node::Documents(docs) => format!("Documents: {} documents", docs.len()),
        Node::Tagged(inner, tag) => format!("Tagged {:?}: {}", tag, node_summary(inner)),
        Node::Anchored(inner, anchor) => format!("Anchored {:?}: {}", anchor, node_summary(inner)),
        Node::Alias(name) => format!("Alias: *{}", name),
        Node::Comment(text) => format!("Comment: # {}", text),
    }
}

/// Pretty-print node structure as a tree
pub fn print_tree(node: &Node) -> String {
    let mut output = String::new();
    print_tree_impl(node, 0, "", &mut output);
    output
}

fn print_tree_impl(node: &Node, depth: usize, prefix: &str, output: &mut String) {
    let indent = "  ".repeat(depth);
    let type_str = node_type(node).as_str();

    output.push_str(&format!("{}{}{}\n", indent, prefix, type_str));

    match node {
        Node::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                print_tree_impl(item, depth + 1, &format!("[{}]: ", i), output);
            }
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs {
                let key_summary = match k {
                    Node::Str(s, _, _) => s.clone(),
                    _ => format!("{:?}", k),
                };
                output.push_str(&format!("{}  {}: ", indent, key_summary));
                print_tree_impl(v, depth + 1, "", output);
            }
        }
        Node::Set(items) => {
            for item in items {
                print_tree_impl(item, depth + 1, "- ", output);
            }
        }
        Node::Document(items) => {
            for item in items {
                print_tree_impl(item, depth + 1, "", output);
            }
        }
        Node::Documents(docs) => {
            for (i, doc) in docs.iter().enumerate() {
                print_tree_impl(doc, depth + 1, &format!("doc[{}]: ", i), output);
            }
        }
        Node::Tagged(inner, tag) => {
            output.push_str(&format!("{}  tag: {:?}\n", indent, tag));
            print_tree_impl(inner, depth + 1, "", output);
        }
        Node::Anchored(inner, anchor) => {
            output.push_str(&format!("{}  anchor: {:?}\n", indent, anchor));
            print_tree_impl(inner, depth + 1, "", output);
        }
        Node::Str(s, _, _) => {
            if s.len() > 40 {
                output.push_str(&format!("{}  \"{}...\"\n", indent, &s[..37]));
            } else {
                output.push_str(&format!("{}  \"{}\"\n", indent, s));
            }
        }
        Node::Number(n) => {
            output.push_str(&format!("{}  {:?}\n", indent, n));
        }
        Node::Boolean(b) => {
            output.push_str(&format!("{}  {}\n", indent, b));
        }
        Node::Alias(name) => {
            output.push_str(&format!("{}  *{}\n", indent, name));
        }
        Node::Comment(text) => {
            output.push_str(&format!("{}  # {}\n", indent, text));
        }
        Node::None => {
            output.push_str(&format!("{}  null\n", indent));
        }
    }
}

/// Find all nodes of a specific type using depth-first traversal
pub fn find_by_type(node: &Node, target_type: NodeType) -> Vec<&Node> {
    node
        .iter_depth_first()
        .filter(|n| node_type(n) == target_type)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type() {
        assert_eq!(node_type(&Node::None), NodeType::Null);
        assert_eq!(node_type(&Node::from(true)), NodeType::Boolean);
        assert_eq!(node_type(&Node::from(42)), NodeType::Integer);
        assert_eq!(node_type(&Node::from("test")), NodeType::String);
        assert_eq!(node_type(&Node::Array(vec![])), NodeType::Array);
    }

    #[test]
    fn test_node_size() {
        assert_eq!(
            node_size(&Node::Array(vec![Node::from(1), Node::from(2)])),
            2
        );
        assert_eq!(node_size(&Node::from("test")), 0);
    }

    #[test]
    fn test_node_depth() {
        let shallow = Node::from("test");
        assert_eq!(node_depth(&shallow), 1);

        let nested = Node::Array(vec![Node::Array(vec![Node::from("deep")])]);
        assert_eq!(node_depth(&nested), 3);
    }

    #[test]
    fn test_node_info() {
        let node = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
        let info = NodeInfo::new(&node);

        assert_eq!(info.node_type, NodeType::Array);
        assert_eq!(info.size, 3);
        assert_eq!(info.depth, 2);
        assert!(!info.has_tag);
        assert!(!info.has_anchor);
    }

    #[test]
    fn test_node_summary() {
        assert_eq!(node_summary(&Node::None), "null");
        assert!(node_summary(&Node::from(true)).contains("Boolean"));
        assert!(node_summary(&Node::from("test")).contains("String"));
    }

    #[test]
    fn test_find_by_type() {
        let tree = Node::Array(vec![
            Node::from("hello"),
            Node::from(42),
            Node::from("world"),
        ]);

        let strings = find_by_type(&tree, NodeType::String);
        assert_eq!(strings.len(), 2);
    }

    #[test]
    fn test_print_tree() {
        let node = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::from(30)),
        ]);

        let tree = print_tree(&node);
        assert!(tree.contains("mapping"));
        assert!(tree.contains("name"));
        assert!(tree.contains("age"));
    }

    #[test]
    fn test_node_type_categories() {
        assert!(NodeType::String.is_scalar());
        assert!(NodeType::Array.is_collection());
        assert!(!NodeType::Array.is_scalar());
    }
}
