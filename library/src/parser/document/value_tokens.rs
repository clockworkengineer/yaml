//! Token-based value parser (proof of concept)
//!
//! This demonstrates how the tokenization approach solves the decorator parsing
//! problem and eliminates infinite loops.

use crate::constants::*;
use crate::error::messages::*;
use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::token_stream::{TokenStream, Decorators};
use crate::parser::lexer::Token;
use crate::parser::document::scalar::parse_scalar;
use crate::parser::directives::DirectiveContext;

/// Try to coerce a value based on a tag
fn try_coerce_tag(tag: &str, node: Node) -> Option<Node> {
    match tag {
        "!!str" | "!str" => {
            let s = match node {
                Node::Str(s, _, _) => s,
                Node::Number(Numeric::Integer(i)) => i.to_string(),
                Node::Number(Numeric::Float(f)) => f.to_string(),
                Node::Boolean(b) => b.to_string(),
                Node::None => String::new(),
                _ => return None,
            };
            Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None))
        }
        "!!int" | "!int" => match node {
            Node::Number(Numeric::Integer(i)) => Some(Node::Number(Numeric::Integer(i))),
            Node::Str(s, _, _) => {
                if let Ok(i) = s.parse::<i64>() {
                    Some(Node::Number(Numeric::Integer(i)))
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!float" | "!float" => match node {
            Node::Number(Numeric::Float(f)) => Some(Node::Number(Numeric::Float(f))),
            Node::Number(Numeric::Integer(i)) => Some(Node::Number(Numeric::Float(i as f64))),
            Node::Str(s, _, _) => {
                if let Ok(f) = s.parse::<f64>() {
                    Some(Node::Number(Numeric::Float(f)))
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!bool" | "!bool" => match node {
            Node::Boolean(b) => Some(Node::Boolean(b)),
            Node::Str(s, _, _) => {
                let sl = s.to_ascii_lowercase();
                match sl.as_str() {
                    "true" | "yes" | "on" => Some(Node::Boolean(true)),
                    "false" | "no" | "off" => Some(Node::Boolean(false)),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Parse a value using tokens (proof of concept)
///
/// This demonstrates how tokenization solves the decorator problem:
/// 1. Decorators are consumed upfront without complex lookahead
/// 2. No infinite loops - tokens have clear boundaries
/// 3. Empty values are naturally supported (EOF after decorator)
/// 4. Both tag+anchor and anchor+tag orderings work
pub fn parse_value_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    // Handle aliases first (they don't have content)
    if matches!(stream.current(), Some(Token::Alias(_))) {
        if let Some(Token::Alias(name)) = stream.current() {
            let alias = name.clone();
            stream.next()?;
            return Ok(Node::Alias(alias));
        }
    }

    // Consume decorators using unified helper (NO INFINITE LOOPS!)
    let decorators = stream.consume_decorators()?;

    // If we have decorators, parse the content
    if decorators.tag.is_some() || decorators.anchor.is_some() {
        // Check what follows decorators - if newline/eof, it's an empty value
        match stream.current() {
            Some(Token::Newline) | Some(Token::Eof) | None => {
                // Decorator on empty value - don't skip newline, return empty
                let mut result = Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None);
                
                if let Some(tag) = decorators.tag {
                    if let Some(coerced) = try_coerce_tag(&tag, result.clone()) {
                        result = coerced;
                    } else {
                        result = Node::Tagged(Box::new(result), tag);
                    }
                }
                
                if let Some(anchor_name) = decorators.anchor {
                    result = Node::Anchored(Box::new(result), anchor_name);
                }
                
                return Ok(result);
            }
            _ => {
                // There's content after decorators, skip whitespace and parse it
                stream.skip_whitespace()?;
            }
        }

        // Parse the actual value content
        let inner = parse_value_content(stream, directives)?;

        // Apply tag coercion first, then wrap with anchor
        let mut result = inner;

        if let Some(tag) = decorators.tag {
            if let Some(coerced) = try_coerce_tag(&tag, result.clone()) {
                result = coerced;
            } else {
                result = Node::Tagged(Box::new(result), tag);
            }
        }

        if let Some(anchor_name) = decorators.anchor {
            if matches!(result, Node::Anchored(_, _)) {
                return Err("A node cannot have multiple anchors".to_string());
            }
            result = Node::Anchored(Box::new(result), anchor_name);
        }

        return Ok(result);
    }

    // No decorators - parse plain value
    parse_value_content(stream, directives)
}

/// Parse value content (the actual value after decorators)
fn parse_value_content(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    match stream.current() {
        Some(Token::FlowMappingStart) => {
            // TODO: call token-based flow mapping parser
            Err("Flow mapping not yet implemented in token parser".to_string())
        }
        Some(Token::FlowSequenceStart) => {
            // TODO: call token-based flow sequence parser
            Err("Flow sequence not yet implemented in token parser".to_string())
        }
        Some(Token::SingleQuoted(s)) | Some(Token::DoubleQuoted(s)) => {
            let content = s.clone();
            stream.next()?;
            Ok(parse_scalar(&content, directives))
        }
        Some(Token::Plain(s)) => {
            let content = s.clone();
            stream.next()?;
            Ok(parse_scalar(&content, directives))
        }
        Some(Token::Dash) => {
            // Nested sequence
            use crate::parser::document::sequence_tokens::parse_sequence_with_tokens;
            parse_sequence_with_tokens(stream, 0, directives)
        }
        Some(Token::Newline) | Some(Token::Indent(_)) => {
            // Decorator followed by newline = empty value (THIS WORKS NOW!)
            Ok(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ))
        }
        Some(Token::Eof) | None => {
            // Decorator at EOF = empty value (THIS WORKS NOW!)
            Ok(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ))
        }
        Some(Token::Alias(name)) => {
            // Alias reference - consume and return
            let alias_name = name.clone();
            stream.next()?;
            Ok(Node::Alias(alias_name))
        }
        Some(token) => Err(format!("Unexpected token in value: {:?}", token)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::io::traits::ISource;

    #[test]
    fn test_tag_on_empty_value() {
        // This is the FH7J pattern that caused infinite loops!
        let mut source = Buffer::new(b"!!str");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();
        
        // Should parse as empty string
        assert!(matches!(result, Node::Str(s, _, _) if s.is_empty()));
    }

    #[test]
    fn test_anchor_on_empty_value() {
        // This is the PW8X pattern that caused infinite loops!
        let mut source = Buffer::new(b"&anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();
        println!("Result: {:?}", result);
        
        // Should parse as anchored empty string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_both_decorators_on_empty() {
        let mut source = Buffer::new(b"!!str &anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();
        
        // Should parse as anchored empty string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_tag_with_plain_value() {
        let mut source = Buffer::new(b"!!str hello");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();
        
        // Should parse as string "hello"
        assert!(matches!(result, Node::Str(s, _, _) if s == "hello"));
    }

    #[test]
    fn test_anchor_with_quoted_value() {
        let mut source = Buffer::new(b"&anchor 'hello'");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();
        
        // Should parse as anchored string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s == "hello"));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_alias() {
        let mut source = Buffer::new(b"*myalias");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();
        
        // Should parse as alias
        assert!(matches!(result, Node::Alias(name) if name == "myalias"));
    }
}
