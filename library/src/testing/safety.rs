//! Memory safety auditing tools
//!
//! Provides utilities for detecting memory leaks, undefined behavior,
//! and other safety issues.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::nodes::node::Node;

/// Memory safety issue
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyIssue {
    /// Potential memory leak
    MemoryLeak(String),
    /// Excessive memory usage
    ExcessiveMemory(usize),
    /// Stack overflow risk
    StackOverflow(usize),
    /// Unbounded allocation
    UnboundedAllocation(String),
    /// Suspicious pointer usage
    SuspiciousPointer(String),
}

/// Memory safety audit result
#[derive(Debug, Clone)]
pub struct SafetyAudit {
    pub issues: Vec<SafetyIssue>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl SafetyAudit {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: SafetyIssue) {
        self.issues.push(issue);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_info(&mut self, info: String) {
        self.info.push(info);
    }

    pub fn is_safe(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }
}

impl Default for SafetyAudit {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit node for safety issues
pub fn audit_node(node: &Node) -> SafetyAudit {
    let mut audit = SafetyAudit::new();

    // Check depth
    let depth = check_depth(node);
    audit.add_info(format!("Maximum depth: {}", depth));
    if depth > 100 {
        audit.add_issue(SafetyIssue::StackOverflow(depth));
    } else if depth > 50 {
        audit.add_warning(format!("Deep nesting ({}), may risk stack overflow", depth));
    }

    // Check size
    let size = count_nodes(node);
    audit.add_info(format!("Total nodes: {}", size));
    if size > 100000 {
        audit.add_issue(SafetyIssue::ExcessiveMemory(size));
    } else if size > 10000 {
        audit.add_warning(format!(
            "Large document ({} nodes), high memory usage",
            size
        ));
    }

    // Check for circular references
    if has_circular_references(node) {
        audit.add_issue(SafetyIssue::MemoryLeak(
            "Circular references detected".to_string(),
        ));
    }

    // Check collection sizes
    check_collection_sizes(node, &mut audit);

    audit
}

/// Check maximum depth of node tree
fn check_depth(node: &Node) -> usize {
    match node {
        Node::Array(items) => 1 + items.iter().map(check_depth).max().unwrap_or(0),
        Node::Mapping(pairs) => {
            1 + pairs
                .iter()
                .map(|(k, v)| check_depth(k).max(check_depth(v)))
                .max()
                .unwrap_or(0)
        }
        Node::Set(items) => 1 + items.iter().map(check_depth).max().unwrap_or(0),
        Node::Document(items) => 1 + items.iter().map(check_depth).max().unwrap_or(0),
        Node::Documents(docs) => 1 + docs.iter().map(check_depth).max().unwrap_or(0),
        Node::Tagged(inner, _) => 1 + check_depth(inner),
        Node::Anchored(inner, _) => 1 + check_depth(inner),
        _ => 1,
    }
}

/// Count total number of nodes
fn count_nodes(node: &Node) -> usize {
    match node {
        Node::Array(items) => 1 + items.iter().map(count_nodes).sum::<usize>(),
        Node::Mapping(pairs) => {
            1 + pairs
                .iter()
                .map(|(k, v)| count_nodes(k) + count_nodes(v))
                .sum::<usize>()
        }
        Node::Set(items) => 1 + items.iter().map(count_nodes).sum::<usize>(),
        Node::Document(items) => 1 + items.iter().map(count_nodes).sum::<usize>(),
        Node::Documents(docs) => 1 + docs.iter().map(count_nodes).sum::<usize>(),
        Node::Tagged(inner, _) => 1 + count_nodes(inner),
        Node::Anchored(inner, _) => 1 + count_nodes(inner),
        _ => 1,
    }
}

/// Check for circular references
fn has_circular_references(node: &Node) -> bool {
    use core::ptr;

    fn check_impl(node: &Node, visited: &mut Vec<*const Node>) -> bool {
        let node_ptr = node as *const Node;

        if visited.iter().any(|&p| ptr::eq(p, node_ptr)) {
            return true;
        }

        visited.push(node_ptr);

        let has_cycle = match node {
            Node::Array(items) => items.iter().any(|n| check_impl(n, visited)),
            Node::Mapping(pairs) => pairs
                .iter()
                .any(|(k, v)| check_impl(k, visited) || check_impl(v, visited)),
            Node::Set(items) => items.iter().any(|n| check_impl(n, visited)),
            Node::Document(items) => items.iter().any(|n| check_impl(n, visited)),
            Node::Documents(docs) => docs.iter().any(|n| check_impl(n, visited)),
            Node::Tagged(inner, _) => check_impl(inner, visited),
            Node::Anchored(inner, _) => check_impl(inner, visited),
            _ => false,
        };

        visited.pop();
        has_cycle
    }

    let mut visited = Vec::new();
    check_impl(node, &mut visited)
}

/// Check collection sizes for unbounded allocations
fn check_collection_sizes(node: &Node, audit: &mut SafetyAudit) {
    match node {
        Node::Array(items) => {
            if items.len() > 10000 {
                audit.add_issue(SafetyIssue::UnboundedAllocation(format!(
                    "Array with {} items",
                    items.len()
                )));
            } else if items.len() > 1000 {
                audit.add_warning(format!("Large array with {} items", items.len()));
            }
            items.iter().for_each(|n| check_collection_sizes(n, audit));
        }
        Node::Mapping(pairs) => {
            if pairs.len() > 10000 {
                audit.add_issue(SafetyIssue::UnboundedAllocation(format!(
                    "Mapping with {} pairs",
                    pairs.len()
                )));
            } else if pairs.len() > 1000 {
                audit.add_warning(format!("Large mapping with {} pairs", pairs.len()));
            }
            pairs.iter().for_each(|(k, v)| {
                check_collection_sizes(k, audit);
                check_collection_sizes(v, audit);
            });
        }
        Node::Set(items) => {
            if items.len() > 10000 {
                audit.add_issue(SafetyIssue::UnboundedAllocation(format!(
                    "Set with {} items",
                    items.len()
                )));
            }
            items.iter().for_each(|n| check_collection_sizes(n, audit));
        }
        Node::Str(s, _, _) => {
            if s.len() > 1000000 {
                audit.add_issue(SafetyIssue::ExcessiveMemory(s.len()));
            }
        }
        Node::Document(items) => {
            items.iter().for_each(|n| check_collection_sizes(n, audit));
        }
        Node::Documents(docs) => {
            docs.iter().for_each(|n| check_collection_sizes(n, audit));
        }
        Node::Tagged(inner, _) => check_collection_sizes(inner, audit),
        Node::Anchored(inner, _) => check_collection_sizes(inner, audit),
        _ => {}
    }
}

/// Memory statistics for a node
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_nodes: usize,
    pub max_depth: usize,
    pub string_count: usize,
    pub total_string_bytes: usize,
    pub array_count: usize,
    pub mapping_count: usize,
    pub estimated_bytes: usize,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            max_depth: 0,
            string_count: 0,
            total_string_bytes: 0,
            array_count: 0,
            mapping_count: 0,
            estimated_bytes: 0,
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate memory statistics for a node
pub fn calculate_memory_stats(node: &Node) -> MemoryStats {
    let mut stats = MemoryStats::new();
    calculate_stats_impl(node, 0, &mut stats);
    stats
}

fn calculate_stats_impl(node: &Node, depth: usize, stats: &mut MemoryStats) {
    stats.total_nodes += 1;
    stats.max_depth = stats.max_depth.max(depth);

    // Rough estimate: 64 bytes per node base
    stats.estimated_bytes += 64;

    match node {
        Node::Str(s, _, _) => {
            stats.string_count += 1;
            stats.total_string_bytes += s.len();
            stats.estimated_bytes += s.len();
        }
        Node::Array(items) => {
            stats.array_count += 1;
            stats.estimated_bytes += items.len() * 8; // Vec overhead
            items
                .iter()
                .for_each(|n| calculate_stats_impl(n, depth + 1, stats));
        }
        Node::Mapping(pairs) => {
            stats.mapping_count += 1;
            stats.estimated_bytes += pairs.len() * 16; // Vec of tuples overhead
            pairs.iter().for_each(|(k, v)| {
                calculate_stats_impl(k, depth + 1, stats);
                calculate_stats_impl(v, depth + 1, stats);
            });
        }
        Node::Set(items) => {
            stats.estimated_bytes += items.len() * 8;
            items
                .iter()
                .for_each(|n| calculate_stats_impl(n, depth + 1, stats));
        }
        Node::Document(items) => {
            items
                .iter()
                .for_each(|n| calculate_stats_impl(n, depth + 1, stats));
        }
        Node::Documents(docs) => {
            docs.iter()
                .for_each(|n| calculate_stats_impl(n, depth + 1, stats));
        }
        Node::Tagged(inner, _) => calculate_stats_impl(inner, depth + 1, stats),
        Node::Anchored(inner, _) => calculate_stats_impl(inner, depth + 1, stats),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::Numeric;

    #[test]
    fn test_check_depth() {
        let shallow = Node::from("test");
        assert_eq!(check_depth(&shallow), 1);

        let nested = Node::Array(vec![Node::Array(vec![Node::from("deep")])]);
        assert_eq!(check_depth(&nested), 3);
    }

    #[test]
    fn test_count_nodes() {
        let single = Node::from("test");
        assert_eq!(count_nodes(&single), 1);

        let array = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
        assert_eq!(count_nodes(&array), 4);
    }

    #[test]
    fn test_audit_simple_node() {
        let node = Node::from("test");
        let audit = audit_node(&node);
        assert!(audit.is_safe());
        assert_eq!(audit.issue_count(), 0);
    }

    #[test]
    fn test_audit_deep_node() {
        // Create a deeply nested structure
        let mut node = Node::from("base");
        for _ in 0..60 {
            node = Node::Array(vec![node]);
        }

        let audit = audit_node(&node);
        // Should have warning about depth
        assert!(!audit.warnings.is_empty() || !audit.issues.is_empty());
    }

    #[test]
    fn test_audit_large_collection() {
        let items: Vec<Node> = (0..2000).map(|i| Node::from(i)).collect();
        let node = Node::Array(items);

        let audit = audit_node(&node);
        // Should have warning about size
        assert!(!audit.warnings.is_empty());
    }

    #[test]
    fn test_memory_stats() {
        let node = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::Number(Numeric::Integer(30))),
        ]);

        let stats = calculate_memory_stats(&node);
        assert!(stats.total_nodes > 0);
        assert!(stats.string_count > 0);
        assert!(stats.mapping_count > 0);
        assert!(stats.estimated_bytes > 0);
    }

    #[test]
    fn test_no_circular_references() {
        let node = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert!(!has_circular_references(&node));
    }

    #[test]
    fn test_memory_stats_complex() {
        let node = Node::Array(vec![
            Node::Mapping(vec![
                (Node::from("a"), Node::from("value")),
                (Node::from("b"), Node::Number(Numeric::Integer(42))),
            ]),
            Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]),
        ]);

        let stats = calculate_memory_stats(&node);
        assert!(stats.total_nodes >= 10);
        assert_eq!(stats.max_depth, 2);
    }
}
