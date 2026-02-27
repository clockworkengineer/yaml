//! Sequence Parsing Logic
//!
//! Implements parsing logic for YAML sequences (arrays), handling sequence items,
//! nested sequences, comments, document boundaries, and indentation tracking.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::parser::ParseResult;
// ...existing code...
use crate::parser::token_stream::TokenStream;
use crate::{combined_loop_guard, loop_guard_init};

/// Parses a YAML sequence (array) with the specified indentation level.
///
/// Processes sequence items marked with '-' at the beginning of lines,
/// handling nested sequences, comments, and document boundaries.
/// Maintains proper indentation tracking for nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The expected indentation level for sequence items
/// Module: parser/document/sequence.rs
use crate::io::traits::ISource;

/// Parses a YAML sequence (array) with the specified indentation level.
#[allow(dead_code)]
pub(crate) fn parse_sequence(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> ParseResult<crate::nodes::node::Node> {
    let mut stream = TokenStream::new(source, directives, false)?;
    // NOTE: parse_sequence is only used in legacy paths, so we pass a default context
    let ctx = crate::parser::utils::context::ParsingContext::default();
    let node = crate::parser::tokens::sequence::parse_sequence_with_tokens(
        &mut stream,
        indent_level,
        indent_level.saturating_sub(1),
        directives,
        &ctx,
        0,
    )?;
    Ok(node)
}

// Helper for nested sequence parsing to avoid double mutable borrow
#[allow(dead_code)]
fn parse_sequence_inner(
    stream: &mut TokenStream,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> crate::parser::ParseResult<crate::nodes::node::Node> {
    let mut items = Vec::new();
    // Guard sequence item count and iteration count using combined loop guard
    loop_guard_init!(sequence_counter);
    while stream.current().is_some() {
        combined_loop_guard!(
            sequence_counter,
            items,
            crate::parser::document::loop_guards::MAX_LOOP_ITERATIONS,
            crate::parser::document::loop_guards::MAX_SEQUENCE_ITEMS,
            "Sequence parsing"
        )?;
        // DRY: skip consecutive newlines and comments upfront
        stream.skip_newlines_and_comments()?;
        match stream.current() {
            Some(crate::parser::lexer::Token::Indent(level)) => {
                if *level < indent_level {
                    break;
                }
                stream.next()?;
                continue;
            }
            Some(crate::parser::lexer::Token::Dash) => {
                stream.next()?;
                stream.skip_whitespace()?;
                match stream.current() {
                    Some(crate::parser::lexer::Token::Dash) => {
                        let nested = parse_sequence_inner(stream, indent_level + 1, directives)?;
                        items.push(nested);
                        continue;
                    }
                    Some(crate::parser::lexer::Token::FlowSequenceStart)
                    | Some(crate::parser::lexer::Token::FlowMappingStart) => {
                        use crate::parser::tokens::value::parse_value_with_tokens;
                        let value = parse_value_with_tokens(stream, directives, 0)?;
                        items.push(value);
                        continue;
                    }
                    _ => {
                        use crate::parser::tokens::value::parse_value_with_tokens;
                        let value = parse_value_with_tokens(stream, directives, 0)?;
                        items.push(value);
                        continue;
                    }
                }
            }
            Some(crate::parser::lexer::Token::Eof)
            | Some(crate::parser::lexer::Token::DocumentEnd)
            | Some(crate::parser::lexer::Token::DocumentStart) => {
                break;
            }
            Some(_) => {
                stream.next()?;
            }
            None => break,
        }
    }
    Ok(crate::nodes::node::Node::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::{BlockStyle, Node, QuoteType};
    use crate::parser::directives::DirectiveContext;

    fn parse_seq_from_str(yaml: &str) -> Node {
        let directives = DirectiveContext::new();
        let mut buf = Buffer::new(yaml.as_bytes());
        parse_sequence(&mut buf, 0, &directives).unwrap()
    }

    #[test]
    fn test_simple_sequence() {
        let node = parse_seq_from_str("- one\n- two\n- three\n");
        assert!(matches!(node, Node::Array(ref arr) if arr.len() == 3));
        if let Node::Array(arr) = node {
            assert_eq!(
                arr[0],
                Node::Str("one".into(), QuoteType::Unquoted, BlockStyle::None)
            );
            assert_eq!(
                arr[1],
                Node::Str("two".into(), QuoteType::Unquoted, BlockStyle::None)
            );
            assert_eq!(
                arr[2],
                Node::Str("three".into(), QuoteType::Unquoted, BlockStyle::None)
            );
        }
    }

    #[test]
    fn test_nested_sequence() {
        let node = parse_seq_from_str("- one\n- - two\n  - three\n- four\n");
        if let Node::Array(arr) = node {
            assert_eq!(arr.len(), 3);
            assert_eq!(
                arr[0],
                Node::Str("one".into(), QuoteType::Unquoted, BlockStyle::None)
            );
            if let Node::Array(nested) = &arr[1] {
                assert_eq!(
                    nested[0],
                    Node::Str("two".into(), QuoteType::Unquoted, BlockStyle::None)
                );
                assert_eq!(
                    nested[1],
                    Node::Str("three".into(), QuoteType::Unquoted, BlockStyle::None)
                );
            } else {
                panic!("Expected nested array");
            }
            assert_eq!(
                arr[2],
                Node::Str("four".into(), QuoteType::Unquoted, BlockStyle::None)
            );
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_empty_sequence() {
        let node = parse_seq_from_str("");
        assert!(matches!(node, Node::Array(ref arr) if arr.is_empty()));
    }

    #[test]
    fn test_sequence_with_comments_and_blank_lines() {
        let node = parse_seq_from_str("\n- one # comment\n\n- two\n  # another comment\n- three\n");
        if let Node::Array(arr) = node {
            assert_eq!(arr.len(), 3);
            assert_eq!(
                arr[0],
                Node::Str("one".into(), QuoteType::Unquoted, BlockStyle::None)
            );
            assert_eq!(
                arr[1],
                Node::Str("two".into(), QuoteType::Unquoted, BlockStyle::None)
            );
            assert_eq!(
                arr[2],
                Node::Str("three".into(), QuoteType::Unquoted, BlockStyle::None)
            );
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_sequence_with_empty_items() {
        let node = parse_seq_from_str("-\n-\n-\n");
        if let Node::Array(arr) = node {
            assert_eq!(arr.len(), 3);
            for item in arr {
                assert_eq!(item, Node::None);
            }
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_sequence_with_quoted_and_unquoted() {
        let node = parse_seq_from_str("- 'a'\n- \"b\"\n- c\n");
        if let Node::Array(arr) = node {
            assert_eq!(
                arr[0],
                Node::Str("a".into(), QuoteType::Single, BlockStyle::None)
            );
            assert_eq!(
                arr[1],
                Node::Str("b".into(), QuoteType::Double, BlockStyle::None)
            );
            assert_eq!(
                arr[2],
                Node::Str("c".into(), QuoteType::Unquoted, BlockStyle::None)
            );
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_sequence_with_flow_sequence() {
        let node = parse_seq_from_str("- [1, 2, 3]\n- four\n");
        if let Node::Array(arr) = node {
            assert_eq!(arr.len(), 2);
            // Only check that the first is an array and the second is a string
            assert!(matches!(&arr[0], Node::Array(_)));
            assert_eq!(
                arr[1],
                Node::Str("four".into(), QuoteType::Unquoted, BlockStyle::None)
            );
        } else {
            panic!("Expected array");
        }
    }

    // #[test]
    // fn test_sequence_error_on_invalid_indent() {
    //     let directives = DirectiveContext::new();
    //     let mut buf = Buffer::new(b"  - one\n- two\n");
    //     let result = parse_sequence(&mut buf, 0, &directives);
    //     if result.is_err() {
    //         // Test passes as expected
    //     } else {
    //         println!(
    //             "test_sequence_error_on_invalid_indent: got result = {:?}",
    //             result
    //         );
    //         panic!("Expected error for invalid indentation, but got Ok");
    //     }
    // }
}
