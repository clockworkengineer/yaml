//! Bridge module for gradual migration from character-based to token-based parsing
//!
//! This module provides adapters that allow mixing character-based and token-based
//! parsing during the migration phase.

use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::token_stream::TokenStream;

/// Parse a value using tokens, wrapping the source in a token stream
///
/// This function serves as a bridge between character-based callers and
/// the token-based value parser.
pub fn parse_value_bridged(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let mut stream = TokenStream::new(source, directives, false)?;

    // Parse using token-based parser (stream is auto-initialized)
    parse_value_with_tokens(&mut stream, directives, 0)
}

/// Check if we should use token-based parsing for a given construct
///
/// During migration, we can selectively enable token-based parsing for
/// specific patterns that benefit most.
///
/// CRITICAL: Avoid using token parsing in contexts where control will return
/// to character-based parsing, as lexer read-ahead causes position sync issues.
pub fn should_use_token_parsing(source: &mut dyn ISource) -> bool {
    // SIMPLIFIED ROUTING: Avoid token/character mixing to prevent position sync issues
    // Only use token parsing for pure flow contexts where we won't return to character parsing

    let uses_tokens = match source.current() {
        Some('*') => {
            // Aliases: Check if in flow context
            let state = source.save_state();
            source.next();
            while let Some(c) = source.current() {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    source.next();
                    continue;
                }
                break;
            }
            // Only use tokens if in flow collection or at EOF
            let in_flow = matches!(source.current(), Some('[') | Some('{') | None);
            source.restore_state(state);
            in_flow
        }
        Some('!') => {
            // DON'T use token parsing for tags - character parser handles all cases
            // This avoids position sync issues when returning to character-based parsing
            false
        }
        Some('&') => {
            // DON'T use token parsing for anchors - character parser handles all cases
            false
        }
        _ => false,
    };

    uses_tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_bridged_tag_on_empty() {
        let mut source = Buffer::new(b"!!str");
        let directives = DirectiveContext::default();

        let result = parse_value_bridged(&mut source, &directives).unwrap();

        assert!(matches!(result, Node::Str(s, _, _) if s.is_empty()));
    }

    #[test]
    fn test_bridged_anchor_on_empty() {
        let mut source = Buffer::new(b"&anchor");
        let directives = DirectiveContext::default();

        let result = parse_value_bridged(&mut source, &directives).unwrap();

        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_bridged_plain_value() {
        let mut source = Buffer::new(b"hello");
        let directives = DirectiveContext::default();

        let result = parse_value_bridged(&mut source, &directives).unwrap();

        assert!(matches!(result, Node::Str(s, _, _) if s == "hello"));
    }
}
