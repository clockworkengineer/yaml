#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::{Node, Numeric};

    #[test]
    fn test_get_document_base_single_document() {
        let doc = Node::Array(vec![Node::Number(Numeric::Integer(42))]);
        let node = Node::Document(Box::new(doc.clone()));
        let result = get_document_base(&node, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &doc);
    }

    #[test]
    fn test_get_document_base_multiple_documents() {
        let doc1 = Node::Array(vec![Node::Number(Numeric::Integer(1))]);
        let doc2 = Node::Array(vec![Node::Number(Numeric::Integer(2))]);
        let node = Node::Array(vec![doc1.clone(), doc2.clone()]);
        let result1 = get_document_base(&node, 0);
        let result2 = get_document_base(&node, 1);
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), &doc1);
        assert_eq!(result2.unwrap(), &doc2);
    }

    #[test]
    fn test_get_document_base_out_of_bounds() {
        let doc1 = Node::Array(vec![Node::Number(Numeric::Integer(1))]);
        let node = Node::Array(vec![doc1.clone()]);
        let result = get_document_base(&node, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_document_base_invalid_node() {
        let node = Node::Number(Numeric::Integer(5));
        let result = get_document_base(&node, 0);
        assert!(result.is_err());
    }
}
