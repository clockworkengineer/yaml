use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::parse_directives;

use helpers::parse_error;
use document_contents::parse_document_contents;
use document_parser::parse_document;

mod anchors;
mod block_scalar;
mod bridge;
mod context;
mod error_builder;
mod document_contents;
mod document_explicit_key;
mod document_parser;
mod helpers;
mod inline;
mod inline_tokens;
mod loop_guards;
mod mapping;
mod mapping_tokens;
mod scalar;
mod sequence;
mod sequence_tokens;
mod tokens;
mod value;
mod value_tokens;

// pub(crate) use helpers::parse_error;
// pub(crate) use document_contents::parse_document_contents;
// pub(crate) use document_parser::parse_document;

/// Main entry point for parsing YAML content from a source.
///
/// Parses one or more YAML documents from the source, handling document
/// separators and creating a Documents node containing all parsed documents.
/// Empty or blank documents are filtered out automatically.
///
/// Also parses directives (%YAML and %TAG) that appear before each document.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// Result containing a Documents Node with all parsed documents or an error string
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    #[cfg(feature = "debug-trace")]
    log::debug!("parse: begin stream");
    let mut docs: Vec<Node> = Vec::new();

    while source.more() {
        // Ensure we're positioned at meaningful content before checks
        crate::utils::skip_whitespace_and_comments(source);
        // Parse directives before this document
        let directives = parse_directives(source)?;

        // Track if we have explicit directives
        let has_explicit_directives =
            directives.yaml_version.is_some() || directives.tag_prefixes.len() > 2;

        // If we have explicit directives, require a following document with content
        if has_explicit_directives {
            let st = source.save_state();
            let mut ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
            // Skip trivia
            ts.skip_whitespace_and_comments()?;
            match ts.current() {
                // A document start marker is acceptable only if followed by real content
                Some(crate::parser::lexer::Token::DocumentStart) => {
                    // Advance past '---' and check for end-of-line content rules in existing logic
                }
                // A document end or EOF immediately after directives is invalid
                Some(crate::parser::lexer::Token::DocumentEnd)
                | Some(crate::parser::lexer::Token::Eof)
                | None => {
                    source.restore_state(st);
                    return Err(parse_error(
                        source,
                        "Directive must be followed by a document",
                    ));
                }
                _ => {
                    // Proceed; there is content following directives
                }
            }
            source.restore_state(st);
        }

        // Check for document start marker (---)
        let has_document_marker = {
            let st = source.save_state();
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
            let res = matches!(
                ts.current(),
                Some(crate::parser::lexer::Token::DocumentStart)
            );
            source.restore_state(st);
            res
        };
        if has_document_marker {
            source.next();
            source.next();
            source.next();

            // After ---, only whitespace/comments, block scalar indicators (|, >), or tags (!) are allowed until end of line
            // crate::utils::skip_whitespace_and_comments(source);
            if let Some(c) = source.current() {
                // Allow: newline, carriage return, comment, block scalar indicators, tags
                if c != '\n'
                    && c != '\r'
                    && c != '#'
                    && c != '>'
                    && c != '|'
                    && c != '!'
                    && c != '-'
                {
                    // Any other content (including mapping keys/values) is invalid on the marker line
                    return Err(helpers::parse_error(
                        source,
                        "YAML 1.2: Document start marker (---) must be on its own line, except for comments, block scalar indicators (|, >), or tags (!). No mapping keys or values allowed on the same line as ---.",
                    ));
                }
            }
            // Skip comments and move to next line if appropriate
            if source.current() == Some('#') {
                helpers::parse_comment(source);
            }
            if source.current() == Some('\n') || source.current() == Some('\r') {
                source.next();
            }
        }

        // Allow explicit directives without a document start marker.
        // Per YAML spec, directives may appear at the top and apply to the following document,
        // with or without an explicit '---'. Do not error here.

        // Parse the document with directive context
        let document = parse_document(source, 0, &directives);
        match document {
            Ok(doc) => {
                // Count all documents, including empty ones, to match stream semantics
                docs.push(doc)
            }
            Err(err) => return Err(err),
        }

        // Check for document end marker (...)
        crate::utils::skip_whitespace_and_comments(source);
        let has_document_end = {
            let st = source.save_state();
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
            let res = matches!(ts.current(), Some(crate::parser::lexer::Token::DocumentEnd));
            source.restore_state(st);
            res
        };
        if has_document_end {
            source.next();
            source.next();
            source.next();

            // Check for invalid content after document end marker
            crate::utils::skip_whitespace_and_comments(source);
            if let Some(c) = source.current() {
                // Allow newline, carriage return, comments, and directives (%)
                if c != '\n' && c != '\r' && c != '#' && c != '%' && c != '-' {
                    // There's non-whitespace, non-comment, non-directive content after ...
                    return Err(parse_error(
                        source,
                        "Invalid content after document end marker (...)",
                    ));
                }
            }

            if source.current() == Some('\n') {
                source.next();
            }
        }

        // If no more content after handling markers, stop
        if !source.more() {
            break;
        }

        // Allow directives to start the next document even without explicit document-end marker.
        // YAML parsers may accept directives at document boundaries without requiring '...'.

        // Continue to parse next document
    }

    if docs.is_empty() {
        docs.push(Document(Vec::new()))
    }
    #[cfg(feature = "debug-trace")]
    log::debug!("parse: end stream with {} document(s)", docs.len());
    Ok(Node::Documents(docs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;
    use crate::parser::document::helpers::parse_comment;
    use crate::parser::document::scalar::parse_scalar_with_tokens;

    use crate::parser::document::inline::{parse_inline_mapping, parse_inline_sequence};
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
        let node = parse_inline_sequence(&mut stream, &directives).unwrap();
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
        let node = parse_inline_sequence(&mut stream, &directives).unwrap();
        assert!(matches!(node, Node::Array(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_inline_mapping_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"{key1: 1, 'key2': \"two\"}");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut src, &directives, false).unwrap();
        let node = parse_inline_mapping(&mut stream, &directives).unwrap();
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
        let node = parse_inline_mapping(&mut stream, &directives).unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_comment_trims_hash_and_newline() {
        let mut src = Buffer::new(b"# Hello world  \n");
        let text = parse_comment(&mut src);
        assert_eq!(text, "Hello world");
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
        let n = parse_document_contents(&mut src, 0, &directives).unwrap();
        assert!(matches!(n, Node::Mapping(_)));
    }
}
