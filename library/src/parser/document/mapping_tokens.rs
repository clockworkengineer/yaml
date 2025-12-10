/// Parse a single key-value mapping pair (for sequence items)
#[allow(dead_code)]
pub fn parse_single_mapping_pair_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let (key, value) = parse_mapping_pair(stream, directives, 0, 0)?;
    Ok(Node::Mapping(vec![(key, value)]))
}
// Token-based mapping parser: Parses YAML mappings using tokenization instead of character-based lookahead. This eliminates infinite loops with decorators and handles complex key patterns.

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Parse a block mapping using tokens
///
/// Example:
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
#[allow(dead_code)]
pub fn parse_mapping_with_tokens(
    stream: &mut TokenStream,
    _base_indent: usize,
    directives: &DirectiveContext,
    depth: usize,
) -> Result<Node, String> {
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_tokens: start parse_mapping_with_tokens");
    let mut pairs = Vec::new();
    // ...existing code...

    // Skip initial whitespace/newlines
    stream.skip_whitespace()?;

    // Track current indentation level
    let mut current_indent: Option<usize> = None;
    let mut first_key = true;

    loop {
        // Skip whitespace/comments before each mapping entry
        while matches!(stream.current(), Some(Token::Newline) | Some(Token::Comment(_))) {
            stream.next()?;
        }
        let token = match stream.current() {
            Some(t) => t,
            None => break,
        };
        // If this is the first key, set indent appropriately
        if first_key {
            match token {
                Token::Indent(level) => {
                    current_indent = Some(*level);
                }
                _ => {
                    current_indent = Some(0);
                }
            }
            first_key = false;
        }
        // If the next token is a Plain token and current_indent > 0, break out
        if let Some(cur) = current_indent {
            if cur > 0 {
                if let Token::Plain(_) = token {
                    break;
                }
            }
        }
        match token {
            Token::Newline | Token::Comment(_) => {
                stream.next()?;
                continue;
            }
            Token::Indent(level) => {
                if let Some(cur) = current_indent {
                    if *level < cur {
                        // Indent decreased: end current mapping
                        break;
                    } else if *level > cur {
                        // Indent increased: skip, let parse_mapping_pair handle as value
                        stream.next()?;
                        continue;
                    } else {
                        stream.next()?;
                        continue;
                    }
                }
            }
            Token::Eof
            | Token::DocumentEnd
            | Token::DocumentStart
            | Token::Dash
            | Token::FlowMappingEnd
            | Token::FlowSequenceEnd => {
                break;
            }
            _ => {
                let cur_indent = current_indent.unwrap_or(0);
                let (key, value) = parse_mapping_pair(stream, directives, cur_indent, depth)?;
                stream.skip_whitespace()?;
                pairs.push((key, value));
                // After adding a pair, if the next token is a plain key and current_indent > 0, break out
                if cur_indent > 0 {
                    if let Some(Token::Plain(_)) = stream.current() {
                        break;
                    }
                }
            }
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!(
        "mapping_tokens: end parse_mapping_with_tokens with {} pair(s)",
        pairs.len()
    );
    Ok(Node::Mapping(pairs))
}

/// Parse a single key-value pair
#[allow(dead_code)]
fn parse_mapping_pair(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    cur_indent: usize,
    depth: usize,
) -> Result<(Node, Node), String> {
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: start at token = {:?}", stream.current());
    // Unify explicit and implicit key handling: always use token stream for key detection
    // If the next token is a question mark, it's an explicit key
    let mut explicit_key = false;
    if matches!(stream.current(), Some(Token::QuestionMark)) {
        stream.next()?;
        stream.skip_whitespace()?;
        explicit_key = true;
    }

    // Handle decorators (tag/anchor) and parse the key value
    let key = {
        let mut parsed_key = if matches!(
            stream.current(),
            Some(Token::Tag(_)) | Some(Token::Anchor(_))
        ) {
            let decorators = stream.consume_decorators()?;
            if matches!(stream.current(), Some(Token::Colon)) {
                use crate::nodes::node::{BlockStyle, QuoteType};
                let mut node = Node::Str("".to_string(), QuoteType::Unquoted, BlockStyle::None);
                if let Some(tag) = decorators.tag {
                    node = Node::Tagged(Box::new(node), tag);
                }
                if let Some(anchor) = decorators.anchor {
                    node = Node::Anchored(Box::new(node), anchor);
                }
                node
            } else {
                parse_value_with_tokens(stream, directives, depth + 1)?
            }
        } else {
            parse_value_with_tokens(stream, directives, depth + 1)?
        };
        // If the key is an array, convert to string representation for mapping key
        if let Node::Array(items) = &parsed_key {
            let mut s = String::from("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                match item {
                    Node::Str(val, _, _) => s.push_str(val),
                    Node::Number(n) => s.push_str(&format!("{:?}", n)),
                    Node::Boolean(b) => s.push_str(&format!("{}", b)),
                    _ => s.push_str(&format!("{:?}", item)),
                }
            }
            s.push(']');
            use crate::nodes::node::{BlockStyle, QuoteType};
            parsed_key = Node::Str(s, QuoteType::Unquoted, BlockStyle::None);
        }
        parsed_key
    };
    match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
        }
        _ if explicit_key => {
            // Explicit key may omit a value entirely (e.g., !!set with '? key').
            // YAML allows explicit keys without a following colon to indicate an
            // empty value. If the colon isn't found on the next non-trivia token,
            // treat the value as empty rather than error.
            if matches!(stream.current(), Some(Token::Newline)) {
                stream.next()?;
                // Skip indentation and comments
                loop {
                    stream.skip_whitespace()?;
                    match stream.current() {
                        Some(Token::Comment(_)) => {
                            stream.next()?;
                            continue;
                        }
                        _ => break,
                    }
                }
                if matches!(stream.current(), Some(Token::Colon)) {
                    stream.next()?;
                } else {
                    // No colon: treat as empty value for explicit key
                    return Ok((key, Node::None));
                }
            } else {
                // No newline after explicit key: if colon absent, treat as empty
                if !matches!(stream.current(), Some(Token::Colon)) {
                    return Ok((key, Node::None));
                } else {
                    stream.next()?;
                }
            }
        }
        Some(Token::Eof) | None => {
            // Treat EOF or None as valid empty value
            return Ok((key, Node::None));
        }
        // If next token is a valid key, treat as empty value
        Some(Token::Plain(_))
        | Some(Token::Tag(_))
        | Some(Token::Anchor(_))
        | Some(Token::QuestionMark)
        | Some(Token::Dash) => {
            return Ok((key, Node::None));
        }
        _ => {
            return Err(format!(
                "Expected colon after key, got: {:?}",
                stream.current()
            ));
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: before value, token = {:?}", stream.current());
    // Parse the value - check for empty value BEFORE skipping whitespace
    let cur_token = stream.current().cloned();
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: value parse branch, cur_token = {:?}", cur_token);
    let value = match cur_token {
        Some(Token::Newline) | None | Some(Token::Eof) => {
            // After colon and newline, check for indent and dash for block sequence value
            // Consume newline
            if matches!(stream.current(), Some(Token::Newline)) {
                stream.next()?;
            }
            // Check for indent
            let indent_level = if let Some(Token::Indent(level)) = stream.current() {
                if *level > cur_indent {
                    let lvl = *level;
                    stream.next()?; // consume Indent
                    Some(lvl)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(level) = indent_level {
                // After indent, skip newlines/comments before checking for dash
                loop {
                    match stream.current() {
                        Some(Token::Newline) | Some(Token::Comment(_)) => { stream.next()?; }
                        _ => break,
                    }
                }
                if matches!(stream.current(), Some(Token::Dash)) {
                    // Parse block sequence as value
                    use crate::parser::document::sequence_tokens::parse_sequence_with_tokens;
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: parsing sequence as value for key {:?}", key);
                    let seq = parse_sequence_with_tokens(stream, level, directives, depth + 1)?;
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: sequence node for key {:?}: {:?}", key, seq);
                    return Ok((key, seq));
                } else {
                    // Parse nested mapping as value
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: parsing mapping as value for key {:?}", key);
                    let map = parse_mapping_with_tokens(stream, level, directives, depth + 1)?;
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: mapping node for key {:?}: {:?}", key, map);
                    return Ok((key, map));
                }
            }
            Node::None
        }
        Some(Token::Indent(level)) => {
            // Increased indentation: parse nested mapping or sequence as value
            stream.next()?; // consume Indent
            if matches!(stream.current(), Some(Token::Dash)) {
                use crate::parser::document::sequence_tokens::parse_sequence_with_tokens;
                parse_sequence_with_tokens(stream, level, directives, depth + 1)?
            } else {
                parse_mapping_with_tokens(stream, level, directives, depth + 1)?
            }
        }
        _ => {
            // Skip whitespace before value
            stream.skip_whitespace()?;
            // Parse the actual value
            let v = parse_value_with_tokens(stream, directives, depth + 1)?;
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: parsed value = {:?}", v);
            v
        }
    };
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
    Ok((key, value))
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
        let node = super::super::inline_tokens::parse_inline_mapping_with_tokens(
            &mut stream,
            &directives,
            0,
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
                    assert_eq!(name, "root");
                    assert!(matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()));
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
}
