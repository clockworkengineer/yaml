
//! Node Tree Traversal Helpers
//!
//! Provides pre-order traversal utilities for YAML node trees, allowing operations
//! to be performed on each node and its children recursively.
//!
//! Copyright (c) 2026 YAML Library Developers


use crate::nodes::node::Node;

/// Pre-order traversal over the YAML `Node` tree.
/// Calls `f` on each node, then recurses into children.
#[allow(dead_code)]
pub fn visit(node: &Node, f: &mut impl FnMut(&Node)) {
    f(node);
    match node {
        Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
            visit(inner, f);
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs {
                visit(k, f);
                visit(v, f);
            }
        }
        Node::Array(items) | Node::Set(items) => {
            for it in items {
                visit(it, f);
            }
        }
        Node::Document(nodes) | Node::Documents(nodes) => {
            for n in nodes {
                visit(n, f);
            }
        }
        _ => {}
    }
}

/// Pre-order traversal over the YAML `Node` tree with depth information.
///
/// Calls `f(node, depth)` on each node, then recurses into children with
/// `depth + 1`. This is useful for read-only analyses that need both the
/// node and its nesting level (e.g., statistics, max-depth calculations),
/// while keeping traversal logic centralized.
#[allow(dead_code)]
pub fn visit_with_depth(node: &Node, depth: usize, f: &mut impl FnMut(&Node, usize)) {
    f(node, depth);
    match node {
        Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
            visit_with_depth(inner, depth + 1, f);
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs {
                visit_with_depth(k, depth + 1, f);
                visit_with_depth(v, depth + 1, f);
            }
        }
        Node::Array(items) | Node::Set(items) => {
            for it in items {
                visit_with_depth(it, depth + 1, f);
            }
        }
        Node::Document(nodes) | Node::Documents(nodes) => {
            for n in nodes {
                visit_with_depth(n, depth + 1, f);
            }
        }
        _ => {}
    }
}

/// Pre-order mutable traversal over the YAML `Node` tree.
/// Calls `f` on each node (allowing mutation), then recurses into children
/// of the potentially mutated node.
#[allow(dead_code)]
pub fn visit_mut(node: &mut Node, f: &mut impl FnMut(&mut Node)) {
    f(node);
    match node {
        Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
            visit_mut(inner.as_mut(), f);
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs.iter_mut() {
                visit_mut(k, f);
                visit_mut(v, f);
            }
        }
        Node::Array(items) | Node::Set(items) => {
            for it in items.iter_mut() {
                visit_mut(it, f);
            }
        }
        Node::Document(nodes) | Node::Documents(nodes) => {
            for n in nodes.iter_mut() {
                visit_mut(n, f);
            }
        }
        _ => {}
    }
}
