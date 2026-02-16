// utils/anchors_helpers.rs
// Helper functions for anchor-related node traversal and error-propagating logic.

use crate::nodes::node::Node;
use crate::parser::ParseResult;

/// Traverses a node tree, applying a closure to each node, propagating errors.
#[allow(dead_code)]
pub fn traverse_with_error<F>(node: &Node, mut f: F) -> ParseResult<()>
where
    F: FnMut(&Node) -> Option<String>,
{
    let mut err: Option<String> = None;
    let mut visit = |n: &Node| {
        if let Some(e) = f(n) {
            err = Some(e);
        }
    };
    crate::parser::utils::visit::visit(node, &mut visit);
    if let Some(e) = err {
        Err(crate::error::YamlError::from(e))
    } else {
        Ok(())
    }
}
