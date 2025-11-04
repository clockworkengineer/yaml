//! Module: misc/mod.rs

use crate::Node;

/// Returns the current version of the package as specified in Cargo.toml.
/// Uses CARGO_PKG_VERSION environment variable that is set during compilation
/// from the version field in Cargo.toml.
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the number of documents in a YAML stream represented by the Documents node.
/// If the node is not a Documents node, returns an error message.
pub fn get_number_of_documents(documents: &Node) -> Result<usize, String> {
    match documents {
        Node::Documents(docs) => Ok(docs.len()),
        _ => Err("Expected Documents node".to_string()),
    }
}
/// Returns the base node of document number n (0-based), reporting any errors.
/// If the node is not a Document or the index is out of bounds, returns an error message.
pub fn get_document_base(node: &Node, n: usize) -> Result<&Node, String> {
    match node {
        Node::Documents(docs) => {
            if n < docs.len() {
                Ok(&docs[n])
            } else {
                Err(format!(
                    "Document index {} out of bounds ({} documents)",
                    n,
                    docs.len()
                ))
            }
        }
        _ => Err("Expected Documents node".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::QuoteType;
    use crate::nodes::node::{BlockStyle, Node};
    use crate::parse;
    #[test]
    fn test_get_version_env() {
        assert_eq!(get_version(), "0.1.0");
    }
    #[test]
    fn test_get_number_of_documents() {
        let mut source = Buffer::new(b"doc1: value1\n---\ndoc2: value2\n---\ndoc3: value3");
        let result = parse(&mut source).unwrap();
        assert_eq!(get_number_of_documents(&result).unwrap(), 3);


        let non_docs_node = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        assert!(get_number_of_documents(&non_docs_node).is_err());
    }

}
