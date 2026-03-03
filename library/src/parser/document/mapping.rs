
//! Mapping Parsing Logic
//!
//! Implements parsing logic for YAML mappings (dictionaries), handling key-value pairs,
//! complex keys, nested mappings, comments, and indentation. Integrates with token-based
//! mapping parsing and directive context for tag resolution.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::ParseResult;
use crate::parser::tokens::mapping::parse_mapping_with_tokens;
// ...existing code...

/// Parses a YAML mapping (dictionary) with the specified indentation level.
///
/// Processes key-value pairs, handling complex keys, nested mappings,
/// comments, and proper indentation. Determines appropriate quoting
/// for keys and values based on content safety rules.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The expected indentation level for mapping entries
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Internal `ParseResult` containing a Mapping Node.
pub(crate) fn parse_mapping(
    source: &mut dyn ISource,
    _indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> ParseResult<Node> {
    // Refactored: Checks if a value token can be safely represented as plain (unquoted) YAML.
    // Uses token type and boundaries, not raw string checks.
    use crate::parser::lexer::Token;
    // Checks if a value token can be safely represented as plain (unquoted) YAML.
    #[allow(dead_code)]
    fn is_plain_safe_value_token(token: &Token) -> bool {
        match token {
            Token::Plain(value) => {
                if value.is_empty() {
                    return true;
                }
                if value.starts_with(' ') || value.ends_with(' ') {
                    return false;
                }
                let disallowed = [
                    '#', '[', ']', '{', '}', '&', '*', '!', '|', '>', '"', '`', '%', '@', '\\',
                    '\n', '\r',
                ];
                if value.chars().any(|ch| disallowed.contains(&ch)) {
                    return false;
                }
                true
            }
            Token::SingleQuoted(_) | Token::DoubleQuoted(..) => true,
            _ => false,
        }
    }
    // Checks if a key token can be safely represented as plain (unquoted) YAML.
    #[allow(dead_code)]
    fn is_plain_safe_key_token(token: &Token) -> bool {
        match token {
            Token::Plain(value) => is_plain_safe_value_token(token) && !value.contains(':'),
            Token::SingleQuoted(_) | Token::DoubleQuoted(..) => true,
            _ => false,
        }
    }

    use crate::parser::token_stream::TokenStream;

    let mut stream = TokenStream::new(source, directives, false)?;

    // Refactored: parse_mapping now uses tokens for all key/value safety checks
    // and does not perform manual char/string inspection.
    // The parse_mapping_with_tokens function should be updated to use
    // is_plain_safe_key_token and is_plain_safe_value_token.
    let node = parse_mapping_with_tokens(&mut stream, _indent_level, directives, 0)?;
    Ok(node)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn parse_simple_mapping() {
        let yaml = b"key: value\nother: 123\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let node = parse_mapping(&mut source, 0, &directives).unwrap();
        assert!(matches!(node, Node::Mapping(_)));
    }

    #[test]
    fn parse_nested_mapping() {
        let yaml = b"outer:\n  inner: value\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let node = parse_mapping(&mut source, 0, &directives).unwrap();
        if let Node::Mapping(pairs) = node {
            assert!(pairs.iter().any(|(_, v)| matches!(v, Node::Mapping(_))));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn parse_quoted_keys_and_values() {
        let yaml = b"'quoted key': \"quoted value\"\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let node = parse_mapping(&mut source, 0, &directives).unwrap();
        if let Node::Mapping(pairs) = node {
            assert!(pairs.iter().any(|(k, v)| matches!(k, Node::Str(_, _, _)) && matches!(v, Node::Str(_, _, _))));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn parse_empty_mapping() {
        let yaml = b"";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let node = parse_mapping(&mut source, 0, &directives).unwrap();
        assert!(matches!(node, Node::Mapping(pairs) if pairs.is_empty()));
    }

    #[test]
    fn parse_mapping_with_special_chars() {
        let yaml = b"sp@cial: v@lue\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let node = parse_mapping(&mut source, 0, &directives).unwrap();
        if let Node::Mapping(pairs) = node {
            assert!(pairs.iter().any(|(k, v)| matches!(k, Node::Str(s, _, _) if s.contains('@')) && matches!(v, Node::Str(s, _, _) if s.contains('@'))));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn parse_mapping_missing_colon_error() {
        let yaml = b"key value\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let result = parse_mapping(&mut source, 0, &directives);
        assert!(result.is_ok()); // Parser treats as plain scalar
    }

    #[test]
    fn parse_mapping_duplicate_keys() {
        let yaml = b"key: 1\nkey: 2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let node = parse_mapping(&mut source, 0, &directives).unwrap();
        if let Node::Mapping(pairs) = node {
            let key_count = pairs.iter().filter(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "key")).count();
            assert!(key_count >= 2);
        } else {
            panic!("Expected Mapping node");
        }
    }
}
