///
/// Basic parsing tests: sequences, mappings, empty documents, comments.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferSource, Node, Node::Document, Numeric, parse};

    #[test]
    fn test_parse_sequence() {
        let mut source = BufferSource::new(b"- 1\n- 2\n- 3");
        let result = parse(&mut source).unwrap();

        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
                Node::Number(Numeric::Integer(3))
            ])])])
        );
    }

    #[test]
    fn test_parse_sequence_with_comments() {
        let mut source = BufferSource::new(b"- 1\n# Comment 1\n- 2\n# Comment 2");
        let result = parse(&mut source).unwrap();

        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2))
            ])])])
        );
    }

    #[test]
    fn test_parse_mapping() {
        let mut source = BufferSource::new(b"key1: value1\nkey2: 42");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(42)),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty() {
        let mut source = BufferSource::new(b"");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_invalid_char() {
        let mut source = BufferSource::new(b"@invalid");
        let result = parse(&mut source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Unexpected character: @"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_parse_comment_only() {
        let mut source = BufferSource::new(b"# Just a comment");
        let result = parse(&mut source).unwrap();

        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_mapping_with_comments() {
        let mut source =
            BufferSource::new(b"# Comment before\nkey1: value1  # Inline comment\n# Comment after");
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);

        assert_eq!(result, expected);
    }
}
