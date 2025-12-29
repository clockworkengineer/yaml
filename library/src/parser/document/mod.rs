///
/// Modules for parsing YAML documents.
///
/// Handles parsing of various YAML constructs including sequences, mappings,
/// scalars, anchors, aliases, and directives. Provides utilities for
/// managing document boundaries and normalization of parsed nodes.
///
mod anchors;
mod block_scalar;
mod bridge;
mod contents;
mod context;
mod error_builder;
mod explicit_key;
mod helpers;
mod inline_tokens;
mod loop_guards;
mod main_loop;
mod mapping;
mod parse;
mod scalar;
mod sequence;
mod tokens;
mod value;

pub use parse::parse;

#[cfg(test)]
mod tests {

    use crate::Node;
    use crate::io::sources::buffer::Buffer;
    use crate::io::traits::ISource;
    use crate::parser::directives::DirectiveContext;
    use crate::parser::document::contents::parse_document_contents;

    use crate::parser::document::scalar::parse_scalar_with_tokens;

    use crate::parser::document::inline_tokens::{parse_inline_mapping_with_tokens, parse_inline_sequence_with_tokens};
    use crate::parser::document::value::parse_value;

    #[test]
    fn test_parse_scalar_with_tokens() {
        use crate::io::sources::buffer::Buffer;
        let directives = crate::parser::directives::DirectiveContext::new();

        // Helper to parse a single-token scalar through TokenStream
        fn parse_one(
            input: &str,
            directives: &crate::parser::directives::DirectiveContext,
        ) -> Result<Node, String> {
            let mut src = Buffer::new(input.as_bytes());
            let mut stream =
                crate::parser::token_stream::TokenStream::new(&mut src, directives, false)?;
            parse_scalar_with_tokens(&mut stream, directives, 0)
        }

        assert_eq!(parse_one("null", &directives), Ok(Node::None));
        assert_eq!(parse_one("~", &directives), Ok(Node::None));
        assert_eq!(parse_one("true", &directives), Ok(Node::Boolean(true)));
        assert_eq!(parse_one("false", &directives), Ok(Node::Boolean(false)));
        assert_eq!(
            parse_one("42", &directives),
            Ok(Node::Number(crate::nodes::node::Numeric::Integer(42)))
        );
        assert_eq!(
            parse_one("3.14", &directives),
            Ok(Node::Number(crate::nodes::node::Numeric::Float(3.14)))
        );
        assert_eq!(
            parse_one("hello", &directives),
            Ok(Node::Str(
                "hello".to_string(),
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None
            ))
        );
        // In token-based parsing, leading '#' starts a comment, so quote to treat as scalar
        assert_eq!(
            parse_one("'#comment'", &directives),
            Ok(Node::Str(
                "#comment".to_string(),
                crate::nodes::node::QuoteType::Single,
                crate::nodes::node::BlockStyle::None
            ))
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_basic() {
        let mut source = Buffer::new(b"key: value");
        assert_eq!(source.get_current_indent_level(), 0);
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_no_colon() {
        let mut source = Buffer::new(b"key value");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            !crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_colon_after_newline() {
        let mut source = Buffer::new(b"key\n: value");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            !crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_spaces_before_colon() {
        let mut source = Buffer::new(b"key   : value");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_empty() {
        let mut source = Buffer::new(b"");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            !crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
    }

    #[test]
    fn test_parse_inline_sequence_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"[1, 'two', 3]");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut src, &directives, false).unwrap();
        let node = parse_inline_sequence_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(matches!(node, Node::Array(_)));
        if let Node::Array(items) = node {
            assert_eq!(items.len(), 3);
            assert!(matches!(
                items[0],
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
            assert!(matches!(
                items[1],
                Node::Str(_, crate::nodes::node::QuoteType::Single, _)
            ));
            assert!(matches!(
                items[2],
                Node::Number(crate::nodes::node::Numeric::Integer(3))
            ));
        }

        let mut empty = Buffer::new(b"[]");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut empty, &directives, false).unwrap();
        let node = parse_inline_sequence_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(matches!(node, Node::Array(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_inline_mapping_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"{key1: 1, 'key2': \"two\"}");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut src, &directives, false).unwrap();
        let node = parse_inline_mapping_with_tokens(&mut stream, &directives, 0, false).unwrap();
        assert!(matches!(node, Node::Mapping(_)));
        if let Node::Mapping(pairs) = node {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(
                pairs[0].0,
                Node::Str(_, crate::nodes::node::QuoteType::Unquoted, _)
            ));
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
            assert!(matches!(
                pairs[1].0,
                Node::Str(_, crate::nodes::node::QuoteType::Single, _)
            ));
            assert!(matches!(pairs[1].1, Node::Str(_, _, _)));
        }

        let mut empty = Buffer::new(b"{}");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut empty, &directives, false).unwrap();
        let node = parse_inline_mapping_with_tokens(&mut stream, &directives, 0, false).unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_value_alias_and_anchor() {
        let directives = DirectiveContext::new();
        let mut a = Buffer::new(b"*myalias");
        let n = parse_value(&mut a, &directives).unwrap();
        assert!(matches!(n, Node::Alias(ref name) if name == "myalias"));

        let mut b = Buffer::new(b"&aname 42");
        let n = parse_value(&mut b, &directives).unwrap();
        if let Node::Anchored(inner, name) = n {
            assert_eq!(*name, "aname".to_string());
            assert!(matches!(
                *inner,
                Node::Number(crate::nodes::node::Numeric::Integer(42))
            ));
        } else {
            panic!("expected Anchored node");
        }
    }

    #[test]
    fn test_parse_document_contents_empty_line() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"key: value\n\n");
        let ctx = crate::parser::document::context::ParsingContext::new(0);
        let n = parse_document_contents(&mut src, 0, &directives, &ctx).unwrap();
        assert!(matches!(n, Node::Mapping(_)));
    }
}
