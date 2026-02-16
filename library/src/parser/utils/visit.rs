//! Module: parser/utils/visit.rs

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
