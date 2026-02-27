
//! Value Parsing Logic
//!
//! Implements parsing logic for YAML values, handling tags, anchors, aliases, inline collections,
//! quoted scalars, and plain scalars. Integrates with directive context for tag resolution and
//! supports type coercion and anchor/alias resolution.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::ParseResult;

/// Validates if a string is valid base64 format

/// Parses a YAML value, handling tags, anchors, aliases, and various value types.
///
/// Processes YAML values including tagged values (!tag), anchored values (&anchor),
/// aliases (*alias), inline collections, quoted scalars, and plain scalars.
/// Handles type coercion based on tags and resolves anchors and aliases.
///
/// Tag handles are resolved using the directive context.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Internal `ParseResult` containing the parsed Node value.
pub(crate) fn parse_value(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> ParseResult<Node> {
    // Selectively route to token-based parser for patterns that benefit:
    // - Decorators on empty values (FH7J, PW8X)
    // - Flow collections (better token boundaries)
    // - Aliases (simpler token handling)

    use crate::parser::tokens::value::parse_value_with_tokens;
    use crate::parser::token_stream::TokenStream;

    let mut stream = TokenStream::new(source, directives, false)?;
    let node = parse_value_with_tokens(&mut stream, directives, 0)?;
    Ok(node)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::directives::DirectiveContext;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::{Node, QuoteType, BlockStyle};

    fn parse_val_from_str(yaml: &str) -> Node {
        let directives = DirectiveContext::new();
        let mut buf = Buffer::new(yaml.as_bytes());
        parse_value(&mut buf, &directives).unwrap()
    }

    #[test]
    fn test_plain_scalar_value() {
        let node = parse_val_from_str("plain value\n");
        assert_eq!(node, Node::Str("plain value".into(), QuoteType::Unquoted, BlockStyle::None));
    }

    #[test]
    fn test_single_quoted_scalar_value() {
        let node = parse_val_from_str("'quoted'\n");
        assert_eq!(node, Node::Str("quoted".into(), QuoteType::Single, BlockStyle::None));
    }

    #[test]
    fn test_double_quoted_scalar_value() {
        let node = parse_val_from_str("\"quoted\"\n");
        assert_eq!(node, Node::Str("quoted".into(), QuoteType::Double, BlockStyle::None));
    }

    #[test]
    fn test_tagged_value() {
        let node = parse_val_from_str("!tag plain\n");
        assert!(matches!(node, Node::Tagged(_, ref tag) if tag == "!tag"));
    }

    #[test]
    fn test_anchored_value() {
        let node = parse_val_from_str("&anchor plain\n");
        assert!(matches!(node, Node::Anchored(_, ref anchor) if anchor == "anchor"));
    }

    #[test]
    fn test_alias_value() {
        let node = parse_val_from_str("*anchor\n");
        assert!(matches!(node, Node::Alias(ref anchor) if anchor == "anchor"));
    }

    #[test]
    fn test_inline_sequence_value() {
        let node = parse_val_from_str("[1, 2, 3]\n");
        assert!(matches!(node, Node::Array(arr) if arr.len() == 3));
    }

    #[test]
    fn test_inline_mapping_value() {
        let node = parse_val_from_str("{a: 1, b: 2}\n");
        assert!(matches!(node, Node::Mapping(pairs) if pairs.len() == 2));
    }
}