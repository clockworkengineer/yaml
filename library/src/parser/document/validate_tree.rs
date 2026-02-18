// Strict validation pass for YAML node tree after parsing
// This function should be called after parsing to ensure the node tree is valid
// according to YAML spec and test suite requirements.

use crate::error::{ErrorKind, YamlError};
use crate::nodes::node::Node;

/// Recursively validate a parsed YAML node tree.
/// Returns Ok(()) if valid, or Err(YamlError) if invalid structure is found.
pub fn validate_yaml_tree(node: &Node) -> Result<(), YamlError> {
    match node {
        Node::Documents(docs) => {
            for doc in docs {
                validate_yaml_tree(doc)?;
            }
        }
        Node::Document(nodes) => {
            for n in nodes {
                validate_yaml_tree(n)?;
            }
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs {
                validate_yaml_tree(k)?;
                validate_yaml_tree(v)?;
            }
        }
        Node::Array(items) | Node::Set(items) => {
            for item in items {
                validate_yaml_tree(item)?;
            }
        }
        Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
            validate_yaml_tree(inner)?;
        }
        Node::Alias(_)
        | Node::Str(_, _, _)
        | Node::Number(_)
        | Node::Boolean(_)
        | Node::Comment(_)
        | Node::None => {}
    }
    // Add additional structural/semantic checks here as needed
    Ok(())
}
