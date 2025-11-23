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
    let mut stream = TokenStream::new(source, directives)?;
    
    // Parse using token-based parser (stream is auto-initialized)
    parse_value_with_tokens(&mut stream, directives)
}

/// Check if we should use token-based parsing for a given construct
///
/// During migration, we can selectively enable token-based parsing for
/// specific patterns that benefit most (e.g., decorators on empty values).
pub fn should_use_token_parsing(source: &mut dyn ISource) -> bool {
    let state = source.save_state();
    
    // Use token parsing ONLY for:
    // 1. Tag followed by newline/colon/flow start (decorator scenarios)
    // 2. Anchor followed by newline/colon/flow start (decorator scenarios)  
    // 3. Aliases (*name) - simpler token handling
    //
    // DON'T use for standalone flow collections - character parser handles those fine
    let uses_tokens = match source.current() {
        Some('*') => {
            // Aliases work well with tokens
            true
        }
        Some('!') => {
            // Skip the tag to collect it
            source.next();
            let mut tag = String::from("!");
            while let Some(c) = source.current() {
                if c == ' ' {
                    source.next();
                    break;
                }
                if c == '!' {
                    tag.push(c);
                    source.next();
                    continue;
                }
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    tag.push(c);
                    source.next();
                    continue;
                }
                break;
            }
            
            // Skip whitespace after tag
            while let Some(c) = source.current() {
                if c == ' ' || c == '\t' {
                    source.next();
                } else {
                    break;
                }
            }
            
            // Special case: !!set with { needs character parser for set syntax
            if (tag == "!!set" || tag == "!set") && source.current() == Some('{') {
                false
            } else {
                // After tag, check if we have newline, colon, or flow collection start
                matches!(source.current(), Some('\n') | Some(':') | Some('[') | Some('{') | None)
            }
        }
        Some('&') => {
            // Skip the anchor to see what follows
            source.next();
            while let Some(c) = source.current() {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    source.next();
                    continue;
                }
                break;
            }
            // After anchor, check for newline, colon, or flow collection start
            matches!(source.current(), Some('\n') | Some(':') | Some('[') | Some('{') | None)
        }
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
