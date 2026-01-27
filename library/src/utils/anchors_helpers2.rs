// Helper functions for anchor lookup, alias replacement, and merge expansion
use crate::nodes::node::Node;
use std::collections::HashMap;

/// Looks up an anchor by name, returning a reference to the node or an error if not found.
pub fn lookup_anchor<'a>(anchors: &'a HashMap<String, Node>, name: &str) -> Result<&'a Node, crate::error::YamlError> {
    anchors.get(name).ok_or_else(|| crate::parser::utils::error_helpers::undefined_anchor(name))
}

/// Checks if a node is a mapping and returns its pairs, or an error if not a mapping.
pub fn as_mapping<'a>(node: &'a Node, name: &str) -> Result<&'a Vec<(Node, Node)>, crate::error::YamlError> {
    if let Node::Mapping(pairs) = node {
        Ok(pairs)
    } else {
        Err(crate::parser::utils::error_helpers::merge_source_not_mapping(name))
    }
}
