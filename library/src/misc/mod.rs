use crate::Node;

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
        _ => Err("Node is not a Document or Array of Documents".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::{BlockStyle, Node};
    use crate::nodes::node::QuoteType;
    use crate::parse;
    #[test]
    fn test_get_number_of_documents() {
        let mut source = Buffer::new(b"doc1: value1\n---\ndoc2: value2\n---\ndoc3: value3");
        let result = parse(&mut source).unwrap();
        assert_eq!(get_number_of_documents(&result).unwrap(), 3);

        // Test error case with non-Documents node
        let non_docs_node = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        assert!(get_number_of_documents(&non_docs_node).is_err());
    }
}
