//! Mapping Tokens & Parsing
//!
//! Contains functions and helpers for parsing YAML mapping tokens and handling
//! indented/nested values, block sequences, and compliance errors.
//!
//! This module is split into focused sub-files included at the bottom:
//!   loop.rs   — main parse_mapping_loop implementation
//!   stack.rs  — indent/dedent stack management and special-token handling
//!   pair.rs   — pair/key/value parsing + apply_decorators_to_key + parse_mapping_value
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::parser::directives::DirectiveContext;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
use crate::parser::tokens::value::parse_value_with_tokens;
use crate::parser::utils::node_utils::force_key_to_string;

/// Context for managing the state of a block mapping parse.
/// Maintains a stack of (indent_level, pairs) to support nested mappings and dedent unwinding.
struct MappingParseContext {
    /// Stack of (indent_level, mapping pairs) for nested mappings
    stack: Vec<(usize, Vec<(Node, Node)>)>,
    /// The base indentation level for this mapping
    base_indent: usize,
    /// True when the very first key of this mapping was parsed inline (on the same
    /// line as a preceding `-` token, i.e. a compact block sequence item).  In that
    /// case the caller supplied base_indent=sequence_indent but the actual key column
    /// may be higher; subsequent keys at the key column are valid and should NOT
    /// trigger the EW3V (wrong indentation) check.
    first_key_was_inline: bool,
}

#[cfg(feature = "debug-trace")]
/// Helper for debug logging of mapping parser internals.
#[inline]
fn mapping_log(msg: String) {
    #[cfg(feature = "std")]
    {
        if let Ok(v) = std::env::var("YAML_TRACE_MAPPING") {
            if v.eq_ignore_ascii_case("1")
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("on")
            {
                log::debug!("{}", msg);
                return;
            }
        }
    }
    log::trace!("{}", msg);
}

/// Parses a value that is indented relative to the current mapping key.
/// Distinguishes between block sequences and nested mappings, and handles YAML compliance errors.
/// Handles indented/nested value after a mapping key.
fn parse_indented_mapping_value(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    cur_indent: usize,
    depth: usize,
    explicit_key: bool,
) -> crate::parser::ParseResult<Node> {
    let indent_level = if let Some(Token::Indent(level)) = stream.current() {
        if *level > cur_indent {
            let _lvl = *level;
            stream.next()?; // consume Indent
            Some(_lvl)
        } else {
            None
        }
    } else {
        None
    };
    if let Some(level) = indent_level {
        stream.skip_newlines_and_comments()?;
        if matches!(stream.current(), Some(Token::Dash)) {
            use crate::parser::tokens::sequence::parse_block_sequence_at;
            let seq = parse_block_sequence_at(stream, level, cur_indent, directives, depth + 1)?;
            return Ok(seq);
        } else {
            let map = parse_mapping_with_tokens(stream, level, directives, depth + 1)?;
            return Ok(map);
        }
    }
    // G9HC: An anchor on its own line at or below the block mapping's indentation
    // level cannot serve as a value for the preceding key.  In YAML, a block
    // mapping value written on a new line must be MORE indented than the key's
    // column.  An anchor at the same (or lower) indentation is neither a valid
    // indented value nor a legitimate standalone mapping key (it lacks ': ').
    //
    // Two sub-cases to detect:
    //   (a) Token::Anchor directly — anchor at column 0 with no Indent token emitted
    //       (only happens when last_indent was already 0 before this line).
    //   (b) Token::Indent(level <= cur_indent) followed by Token::Anchor — anchor at
    //       the same indentation as the current block mapping frame.
    if indent_level.is_none() {
        let is_direct_anchor = matches!(stream.current(), Some(Token::Anchor(_)));
        let is_shallow_indent_before_anchor = match stream.current() {
            Some(Token::Indent(lvl)) => {
                let lvl = *lvl;
                lvl <= cur_indent && matches!(stream.peek()?, Some(Token::Anchor(_)))
            }
            _ => false,
        };
        if is_direct_anchor || is_shallow_indent_before_anchor {
            return Err(crate::parser::utils::error_builder::syntax_error(
                stream.source_mut(),
                "Anchor on its own line at block mapping indentation \
                 without a valid indented value (zero-indented anchor)",
            ));
        }
    }
    // YAML compliance error: Mapping key without value (expected value after colon)
    if !explicit_key && matches!(stream.current(), Some(Token::Eof) | None) {
        let err = crate::parser::errors::mapping_errors::
            mapping_key_without_value_expected_value_after_colon(stream);
        return Err(err.to_string().into());
    }
    Ok(Node::None)
}

/// Parses a single key-value mapping pair (for sequence items).
/// Used when a mapping pair appears as a sequence item (e.g., - key: value).
#[allow(dead_code)]
pub fn parse_single_mapping_pair_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> crate::parser::ParseResult<Node> {
    let ctx = MappingParseContext {
        stack: vec![(0, Vec::new())],
        base_indent: 0,
        first_key_was_inline: false,
    };
    let (key, value) = ctx.parse_mapping_pair(stream, directives, 0, 0)?;
    Ok(Node::Mapping(vec![(key, value)]))
}

/// Parses a block mapping using tokens.
/// This is the main entry point for block mapping parsing in the token-based parser.
/// Handles indentation, dedent unwinding, and special YAML tokens.
///
/// # Example
/// ```yaml
/// key1: value1
/// key2: value2
/// !!str: tagged_key
/// ? complex_key
/// : complex_value
/// ```
///
/// Benefits of token-based approach:
/// - No complex lookahead for keys with decorators
/// - Clear token boundaries prevent infinite loops
/// - Natural handling of explicit keys (?)
pub fn parse_mapping_with_tokens(
    stream: &mut TokenStream,
    base_indent: usize,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    use crate::utils::optimization::{CapacityHints, NodeBuilder};
    let node_builder = NodeBuilder::with_hints(CapacityHints::small());
    let mut ctx = MappingParseContext {
        stack: vec![(
            base_indent,
            Vec::with_capacity(node_builder.hints().mapping_pairs),
        )],
        base_indent,
        first_key_was_inline: false,
    };

    stream.skip_trivia()?;
    ctx.parse_mapping_loop(stream, directives, depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_simple_mapping() {
        let yaml = b"key1: value1\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Mapping node");
        }
    }

    // Note: U44R is covered via the YAML test suite; document-level
    // validation is applied narrowly to avoid false positives in
    // free-form mappings used across examples.

    #[test]
    fn debug_8xdj_mapping_tokens() {
        // 8XDJ: comment inside what should be a plain multiline value
        let yaml = b"key: word1\n#  xxx\n  word2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0);
        assert!(
            result.is_err(),
            "8XDJ mapping via tokens should be rejected as invalid, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_mapping_with_empty_value() {
        let yaml = b"key1:\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_mapping_with_decorated_key() {
        let yaml = b"!!str: value\n&anchor: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged empty string
            // Second key should be anchored
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_fh7j_nested_mapping() {
        // FH7J has: "  !!null : a\n  b: !!str\n"
        let yaml = b"!!null: a\nb: !!str";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged null (empty)
            // Second value should be tagged empty string
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_keys_block_mapping() {
        // Explicit keys without values should produce Node::None values
        let yaml = b"? item1\n? item2\n? item3\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 3);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::None));
            assert!(matches!(pairs[2].1, Node::None));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_key_with_value() {
        // Explicit key followed by value on same line
        let yaml = b"? key1: value1\n? key2\n: value2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // key1 has value1; key2 should have value2
            assert!(matches!(pairs[0].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[0].1, Node::Str(ref s, _, _) if s == "value1"));
            assert!(matches!(pairs[1].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[1].1, Node::Str(ref s, _, _) if s == "value2"));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_complex_key_array() {
        // Complex explicit key (array) should normalize to string key
        let yaml = b"? [a, b, c]: 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Key should be a string representation of the array
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("a") && s.contains("b") && s.contains("c"))
            );
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_mapping() {
        let yaml = b"{}\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        // Inline empty mapping should parse via inline_tokens, but base parser should gracefully handle
        let node = crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
            &mut stream,
            &directives,
            0,
            false,
            None,
        )
        .unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_multiline_key_value_mapping() {
        // Multiline plain scalar key and value using block scalar-like lines
        let yaml = b"? |\n  multi\n  line\n: |\n  val\n  ue\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Keys/values produced by scalar parser should be strings (literal preserves newlines)
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("multi") && s.contains("line"))
            );
            assert!(
                matches!(pairs[0].1, Node::Str(ref s, _, _) if s.contains("val") && s.contains("ue"))
            );
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_value_on_same_line_and_next_line() {
        let yaml = b"key1: \nkey2:\n  - 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::Array(_)));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_decorated_empty_keys_tag_and_anchor() {
        // Decorated empty keys should produce empty-string keys wrapped by tag/anchor
        let yaml = b"!!str: one\n&root: two\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key is tagged empty string
            match &pairs[0].0 {
                Node::Tagged(inner, tag) => {
                    assert!(matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()));
                    assert!(tag.starts_with("!!") || tag.starts_with("!"));
                }
                other => panic!("Expected Tagged empty key, got {:?}", other),
            }
            // Second key is anchored empty string
            match &pairs[1].0 {
                Node::Anchored(inner, name) => {
                    // Allow anchors that decorate either an empty key or a
                    // scalar key; the main requirement is that the anchor
                    // name itself matches and that the parser preserves the
                    // anchoring semantics.
                    assert_eq!(name, "root");
                    assert!(matches!(**inner, Node::Str(_, _, _)));
                }
                other => panic!("Expected Anchored empty key, got {:?}", other),
            }
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_keys_with_nested_sequence_values() {
        // Explicit keys followed by nested sequences
        let yaml = b"? key1\n: \n  - a\n  - b\n? key2\n: \n  - 1\n  - 2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::Array(ref v) if v.len() == 2));
            assert!(matches!(pairs[1].1, Node::Array(ref v) if v.len() == 2));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_zcz6_plain_value_with_nested_colon_should_error() {
        // ZCZ6: `a: b: c: d` — plain value followed by ':' on same line
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config("a: b: c: d\n", config);
        assert!(
            result.is_err(),
            "ZCZ6 should fail: 'a: b: c: d' has ambiguous nested ':' on same line, got: {:?}",
            result
        );
    }

    #[test]
    fn test_k858_block_scalar_chomping_should_succeed() {
        // K858: Spec Example 8.6 - block scalars with chomping indicators
        let yaml = "strip: >-\n\nclip: >\n\nkeep: |+\n\n\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_ok(),
            "K858 should succeed: block scalars with chomping indicators, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_v9d5_compact_block_mappings_should_succeed() {
        // V9D5: Spec Example 8.19 - compact block mappings
        // Note: use CRLF line endings as in the official test suite file
        let yaml = "- sun: yellow\r\n- ? earth: blue\r\n  : moon: white\r\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_ok(),
            "V9D5 should succeed: compact block mappings, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_ew3v_wrong_indentation_in_mapping_should_error() {
        // EW3V: "Wrong indentation in mapping"
        // k1 is at indent 0, k2 is at indent 1 — keys at different indent levels in
        // the same block mapping is invalid (YAML §8.1).
        let yaml = "k1: v1\n k2: v2\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "EW3V should fail: second key more indented than first key in same mapping, got: {:?}",
            result
        );
    }

    #[test]
    fn test_g7je_multiline_implicit_key_should_error() {
        // G7JE: "Multiline implicit keys"
        // Input: `a\nb: 1\r\nc\r\n d: 1\r\n`
        // `c` is an implicit key at indent 0 with no `:` on its line.
        // ` d: 1` at indent 1 follows — a deeper-indented continuation
        // without from an implicit key having colon on the same line.
        // YAML §8.1.1: implicit block keys must be single-line.
        let yaml = "a\\nb: 1\r\nc\r\n d: 1\r\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "G7JE should fail: implicit key followed by deeper-indented content without colon, got: {:?}",
            result
        );
    }

    #[test]
    fn test_eb22_directive_after_content_without_doc_end_should_error() {
        // EB22: "Missing document-end marker before directive"
        // A %YAML directive appears after document content but without a preceding
        // document-end marker (...). This is invalid per YAML spec.
        let yaml = "---\r\nscalar1 # comment\r\n%YAML 1.2\r\n---\r\nscalar2\r\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "EB22 should fail: %YAML directive after content without document-end marker, got: {:?}",
            result
        );
    }

    #[test]
    fn test_dk95_06_tab_after_spaces_indentation_should_error() {
        // DK95/06: "Tabs that look like indentation"
        // Input: foo:\r\n  a: 1\r\n  \tb: 2\r\n
        // The line "  \tb: 2" has 2 spaces + tab before the mapping key "b".
        // This is a tab used as indentation which is invalid in YAML block context.
        let yaml = "foo:\r\n  a: 1\r\n  \tb: 2\r\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "DK95/06 should fail: tab after spaces in indentation before mapping key, got: {:?}",
            result
        );
    }

    #[test]
    fn test_g9hc_anchor_at_mapping_indentation_should_error() {
        // G9HC: an anchor on its own line at the block mapping's indentation
        // level (zero-indented in this case) is invalid — the sequence that
        // follows is not indented enough to be the anchor's decorated value
        // for the 'seq:' key (which requires indentation > 0).
        let yaml = "---\nseq:\n&anchor\n- a\n- b\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "G9HC should fail: anchor at block mapping level without valid indented value, got: {:?}",
            result
        );
    }

    #[test]
    fn test_jkf3_multiline_unindented_double_quoted_block_key_should_error() {
        // JKF3: "Multiline unidented double quoted block key"
        // Input: `- - "bar\r\nbar": x`
        // The double-quoted key spans two source lines.  The continuation `bar"` is at
        // column 0 — less indented than the enclosing block sequence context.
        // YAML §8.1.1: implicit block mapping keys must fit on a single line.
        let yaml = "- - \"bar\r\nbar\": x\r\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "JKF3 should fail: multiline double-quoted implicit block key, got: {:?}",
            result
        );
    }

    #[test]
    fn test_ks4u_invalid_item_after_end_of_flow_sequence_should_error() {
        // KS4U: "Invalid item after end of flow sequence"
        // Input: `---\n[\nsequence item\n]\ninvalid item\n`
        // After the flow sequence (the document value) is closed with `]`,
        // `invalid item` appears on a new line.  A document can have only one
        // root node; trailing plain-scalar content after the closing `]` is invalid.
        let yaml = "---\r\n[\r\nsequence item\r\n]\r\ninvalid item\r\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "KS4U should fail: plain scalar after closing ']' of top-level flow sequence, got: {:?}",
            result
        );
    }
}

// ---------------------------------------------------------------------------
// Additional impl blocks for MappingParseContext — included directly into
// this module so all private struct fields remain accessible.
// ---------------------------------------------------------------------------
include!("loop.rs");
include!("stack.rs");
include!("pair.rs");
