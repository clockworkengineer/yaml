//! Node diffing utilities
//!
//! Provides tools for comparing YAML nodes and identifying differences.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::nodes::node::Node;

/// Type of difference between nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    /// Node was added
    Added,
    /// Node was removed
    Removed,
    /// Node was modified
    Modified,
    /// Node type changed
    TypeChanged,
    /// Collection size changed
    SizeChanged,
}

/// A difference between two nodes
#[derive(Debug, Clone)]
pub struct Diff {
    pub diff_type: DiffType,
    pub path: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub description: String,
}

impl Diff {
    /// Create a new diff
    pub fn new(diff_type: DiffType, path: String, description: String) -> Self {
        Self {
            diff_type,
            path,
            old_value: None,
            new_value: None,
            description,
        }
    }

    /// Create with old and new values
    pub fn with_values(
        diff_type: DiffType,
        path: String,
        old_value: Option<String>,
        new_value: Option<String>,
        description: String,
    ) -> Self {
        Self {
            diff_type,
            path,
            old_value,
            new_value,
            description,
        }
    }

    /// Format as a readable string
    pub fn format(&self) -> String {
        let mut result = format!("[{:?}] {}: {}", self.diff_type, self.path, self.description);

        if let Some(old) = &self.old_value {
            result.push_str(&format!("\n  Old: {}", old));
        }

        if let Some(new) = &self.new_value {
            result.push_str(&format!("\n  New: {}", new));
        }

        result
    }
}

/// Result of a diff operation
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub diffs: Vec<Diff>,
    pub identical: bool,
}

impl DiffResult {
    /// Create a new diff result
    pub fn new() -> Self {
        Self {
            diffs: Vec::new(),
            identical: true,
        }
    }

    /// Add a diff
    pub fn add_diff(&mut self, diff: Diff) {
        self.identical = false;
        self.diffs.push(diff);
    }

    /// Check if nodes are identical
    pub fn is_identical(&self) -> bool {
        self.identical
    }

    /// Get number of differences
    pub fn count(&self) -> usize {
        self.diffs.len()
    }

    /// Format all diffs as a string
    pub fn format(&self) -> String {
        if self.identical {
            "No differences found".to_string()
        } else {
            let mut result = format!("Found {} difference(s):\n", self.count());
            for diff in &self.diffs {
                result.push_str(&diff.format());
                result.push('\n');
            }
            result
        }
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare two nodes and find differences
pub fn diff_nodes(old: &Node, new: &Node) -> DiffResult {
    let mut result = DiffResult::new();
    diff_nodes_impl(old, new, String::new(), &mut result);
    result
}

fn diff_nodes_impl(old: &Node, new: &Node, path: String, result: &mut DiffResult) {
    use crate::devtools::inspect::{node_summary, node_type};

    let old_type = node_type(old);
    let new_type = node_type(new);

    // Check type difference
    if old_type != new_type {
        result.add_diff(Diff::with_values(
            DiffType::TypeChanged,
            path.clone(),
            Some(old_type.as_str().to_string()),
            Some(new_type.as_str().to_string()),
            format!("Type changed from {:?} to {:?}", old_type, new_type),
        ));
        return;
    }

    // Check value differences based on type
    match (old, new) {
        (Node::None, Node::None) => {}
        (Node::Boolean(o), Node::Boolean(n)) if o == n => {}
        (Node::Number(o), Node::Number(n)) if o == n => {}
        (Node::Str(o, _, _), Node::Str(n, _, _)) if o == n => {}

        (Node::Boolean(o), Node::Boolean(n)) => {
            result.add_diff(Diff::with_values(
                DiffType::Modified,
                path,
                Some(o.to_string()),
                Some(n.to_string()),
                "Boolean value changed".to_string(),
            ));
        }

        (Node::Number(o), Node::Number(n)) => {
            result.add_diff(Diff::with_values(
                DiffType::Modified,
                path,
                Some(format!("{:?}", o)),
                Some(format!("{:?}", n)),
                "Number value changed".to_string(),
            ));
        }

        (Node::Str(o, _, _), Node::Str(n, _, _)) => {
            result.add_diff(Diff::with_values(
                DiffType::Modified,
                path,
                Some(o.clone()),
                Some(n.clone()),
                "String value changed".to_string(),
            ));
        }

        (Node::Array(old_items), Node::Array(new_items)) => {
            if old_items.len() != new_items.len() {
                result.add_diff(Diff::with_values(
                    DiffType::SizeChanged,
                    path.clone(),
                    Some(old_items.len().to_string()),
                    Some(new_items.len().to_string()),
                    "Array size changed".to_string(),
                ));
            }

            let min_len = old_items.len().min(new_items.len());
            for i in 0..min_len {
                let item_path = format!("{}[{}]", path, i);
                diff_nodes_impl(&old_items[i], &new_items[i], item_path, result);
            }

            // Handle extra items
            for i in min_len..old_items.len() {
                let item_path = format!("{}[{}]", path, i);
                result.add_diff(Diff::with_values(
                    DiffType::Removed,
                    item_path,
                    Some(node_summary(&old_items[i])),
                    None,
                    "Array item removed".to_string(),
                ));
            }

            for i in min_len..new_items.len() {
                let item_path = format!("{}[{}]", path, i);
                result.add_diff(Diff::with_values(
                    DiffType::Added,
                    item_path,
                    None,
                    Some(node_summary(&new_items[i])),
                    "Array item added".to_string(),
                ));
            }
        }

        (Node::Mapping(old_pairs), Node::Mapping(new_pairs)) => {
            if old_pairs.len() != new_pairs.len() {
                result.add_diff(Diff::with_values(
                    DiffType::SizeChanged,
                    path.clone(),
                    Some(old_pairs.len().to_string()),
                    Some(new_pairs.len().to_string()),
                    "Mapping size changed".to_string(),
                ));
            }

            // Compare common keys
            for (old_key, old_val) in old_pairs {
                let key_str = match old_key {
                    Node::Str(s, _, _) => s.clone(),
                    _ => format!("{:?}", old_key),
                };

                let new_val = new_pairs.iter().find(|(k, _)| k == old_key).map(|(_, v)| v);

                let item_path = if path.is_empty() {
                    key_str.clone()
                } else {
                    format!("{}.{}", path, key_str)
                };

                match new_val {
                    Some(nv) => diff_nodes_impl(old_val, nv, item_path, result),
                    None => {
                        result.add_diff(Diff::with_values(
                            DiffType::Removed,
                            item_path,
                            Some(node_summary(old_val)),
                            None,
                            "Mapping key removed".to_string(),
                        ));
                    }
                }
            }

            // Check for added keys
            for (new_key, new_val) in new_pairs {
                let exists = old_pairs.iter().any(|(k, _)| k == new_key);
                if !exists {
                    let key_str = match new_key {
                        Node::Str(s, _, _) => s.clone(),
                        _ => format!("{:?}", new_key),
                    };

                    let item_path = if path.is_empty() {
                        key_str.clone()
                    } else {
                        format!("{}.{}", path, key_str)
                    };

                    result.add_diff(Diff::with_values(
                        DiffType::Added,
                        item_path,
                        None,
                        Some(node_summary(new_val)),
                        "Mapping key added".to_string(),
                    ));
                }
            }
        }

        _ => {
            // Fallback for other types
            if node_summary(old) != node_summary(new) {
                result.add_diff(Diff::with_values(
                    DiffType::Modified,
                    path,
                    Some(node_summary(old)),
                    Some(node_summary(new)),
                    "Value changed".to_string(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_nodes() {
        let node1 = Node::from("test");
        let node2 = Node::from("test");

        let result = diff_nodes(&node1, &node2);
        assert!(result.is_identical());
        assert_eq!(result.count(), 0);
    }

    #[test]
    fn test_different_strings() {
        let node1 = Node::from("old");
        let node2 = Node::from("new");

        let result = diff_nodes(&node1, &node2);
        assert!(!result.is_identical());
        assert_eq!(result.count(), 1);
        assert_eq!(result.diffs[0].diff_type, DiffType::Modified);
    }

    #[test]
    fn test_type_change() {
        let node1 = Node::from("test");
        let node2 = Node::from(42);

        let result = diff_nodes(&node1, &node2);
        assert!(!result.is_identical());
        assert_eq!(result.diffs[0].diff_type, DiffType::TypeChanged);
    }

    #[test]
    fn test_array_size_change() {
        let node1 = Node::Array(vec![Node::from(1), Node::from(2)]);
        let node2 = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        let result = diff_nodes(&node1, &node2);
        assert!(!result.is_identical());
        assert!(
            result
                .diffs
                .iter()
                .any(|d| d.diff_type == DiffType::SizeChanged)
        );
        assert!(result.diffs.iter().any(|d| d.diff_type == DiffType::Added));
    }

    #[test]
    fn test_mapping_key_added() {
        let node1 = Node::Mapping(vec![(Node::from("name"), Node::from("Alice"))]);
        let node2 = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::from(30)),
        ]);

        let result = diff_nodes(&node1, &node2);
        assert!(!result.is_identical());
        assert!(result.diffs.iter().any(|d| d.diff_type == DiffType::Added));
    }

    #[test]
    fn test_mapping_key_removed() {
        let node1 = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::from(30)),
        ]);
        let node2 = Node::Mapping(vec![(Node::from("name"), Node::from("Alice"))]);

        let result = diff_nodes(&node1, &node2);
        assert!(!result.is_identical());
        assert!(
            result
                .diffs
                .iter()
                .any(|d| d.diff_type == DiffType::Removed)
        );
    }

    #[test]
    fn test_diff_format() {
        let diff = Diff::with_values(
            DiffType::Modified,
            "test.value".to_string(),
            Some("old".to_string()),
            Some("new".to_string()),
            "Value changed".to_string(),
        );

        let formatted = diff.format();
        assert!(formatted.contains("Modified"));
        assert!(formatted.contains("test.value"));
        assert!(formatted.contains("old"));
        assert!(formatted.contains("new"));
    }
}
