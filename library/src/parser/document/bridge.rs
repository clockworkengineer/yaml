//! Bridge module for gradual migration from character-based to token-based parsing
//!
//! This module provides adapters that allow mixing character-based and token-based
//! parsing during the migration phase.

use crate::io::traits::ISource;
use crate::parser::token_stream::TokenStream;
use crate::parser::directives::DirectiveContext;
use crate::nodes::node::Node;
use crate::parser::document::value_tokens::parse_value_with_tokens;

/// Parse a value using tokens, wrapping the source in a token stream
///
/// This function serves as a bridge between character-based callers and
/// the token-based value parser.
pub fn parse_value_bridged(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let mut stream = TokenStream::new(source, directives);
    
    // Initialize the token stream
    stream.next()?;
    
    // Parse using token-based parser
    parse_value_with_tokens(&mut stream, directives)
}

/// Check if we should use token-based parsing for a given construct
///
/// During migration, we can selectively enable token-based parsing for
/// specific patterns that benefit most (e.g., decorators).
pub fn should_use_token_parsing(source: &mut dyn ISource) -> bool {
    let state = source.save_state();
    
    // Check if this looks like a decorator pattern (tag or anchor)
    let uses_tokens = match source.current() {
        Some('!') | Some('&') | Some('*') => true,
        _ => false,
    };
    
    source.restore_state(state);
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
