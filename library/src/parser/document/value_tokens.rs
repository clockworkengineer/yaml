//! Token-based value parser (proof of concept)
//!
//! This demonstrates how the tokenization approach solves the decorator parsing
//! problem and eliminates infinite loops.

use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::directives::DirectiveContext;
use crate::parser::document::error_builder::{structure_error, syntax_error};
// ...existing code...
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

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
        "!!set" | "!set" | "tag:yaml.org,2002:set" => match node {
            // Convert mapping with null values to a set
            Node::Mapping(pairs) => {
                let mut set_items = Vec::new();
                for (key, value) in pairs {
                    match value {
                        Node::None => {
                            set_items.push(key);
                        }
                        _ => return None, // Not a valid set mapping
                    }
                }
                Some(Node::Set(set_items))
            }
            // Convert array to a set (remove duplicates)
            Node::Array(items) => {
                let mut unique_items = Vec::new();
                for item in items {
                    if !unique_items.contains(&item) {
                        unique_items.push(item);
                    }
                }
                Some(Node::Set(unique_items))
            }
            // Single value becomes a set with one element
            _ => Some(Node::Set(vec![node])),
        },
        "!!omap" | "!omap" | "tag:yaml.org,2002:omap" => match node {
            // Ordered mapping - preserve as tagged array of single-key mappings
            Node::Array(items) => {
                // Validate that each item is a mapping with one key-value pair
                for item in &items {
                    match item {
                        Node::Mapping(pairs) if pairs.len() == 1 => {}
                        _ => return None, // Invalid omap format
                    }
                }
                Some(Node::Tagged(
                    Box::new(Node::Array(items)),
                    "tag:yaml.org,2002:omap".to_string(),
                ))
            }
            _ => None,
        },
        "!!pairs" | "!pairs" | "tag:yaml.org,2002:pairs" => match node {
            // Pairs - preserve as tagged array
            Node::Array(items) => Some(Node::Tagged(
                Box::new(Node::Array(items)),
                "tag:yaml.org,2002:pairs".to_string(),
            )),
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
        // Skip whitespace/newlines after decorators to find the actual content
        stream.skip_whitespace()?;

        // NOW check if we're at EOF or end of structure (empty decorated value)
        // This includes:
        // - EOF/None: end of document
        // - Dash: next sequence item (e.g., "- !!str\n-" = empty tagged value)
        // - Colon: mapping key (e.g., "!!str :")
        // - FlowMappingEnd/FlowSequenceEnd: end of flow collection
        // - DocumentStart/DocumentEnd: document boundary
        match stream.current() {
            Some(Token::Eof)
            | Some(Token::Dash)
            | Some(Token::Colon)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd)
            | Some(Token::DocumentStart)
            | Some(Token::DocumentEnd)
            | None => {
                // Decorator with no content - empty value
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
                // There's content after decorators, continue to parse it
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
                return Err(structure_error(
                    stream.source_mut(),
                    "A node cannot have multiple anchors",
                ));
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
    // Skip comments before parsing value
    while matches!(stream.current(), Some(Token::Comment(_))) {
        stream.next()?;
    }
    match stream.current() {
        Some(Token::FlowMappingStart) => {
            use crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens;
            parse_inline_mapping_with_tokens(stream, directives)
        }
        Some(Token::FlowSequenceStart) => {
            use crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens;
            parse_inline_sequence_with_tokens(stream, directives)
        }
        Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(_)) | Some(Token::Plain(_)) => {
            crate::parser::document::scalar::parse_scalar_with_tokens(stream, directives, 0)
        }
        Some(Token::Dash) => {
            use crate::parser::document::sequence_tokens::parse_sequence_with_tokens;
            parse_sequence_with_tokens(stream, 0, directives)
        }
        Some(Token::Indent(level)) => {
            // Indented value: parse nested mapping
            use crate::parser::document::mapping_tokens::parse_mapping_with_tokens;
            parse_mapping_with_tokens(stream, *level, directives)
        }
        Some(Token::Newline) => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(Token::Eof) | None => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(Token::Alias(name)) => {
            let alias_name = name.clone();
            stream.next()?;
            Ok(Node::Alias(alias_name))
        }
        Some(Token::Colon) => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(token) => {
            let token_str = format!("Unexpected token in value: {:?}", token);
            Err(syntax_error(stream.source_mut(), &token_str))
        }
    }
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_tag_on_empty_value() {
        // This is the FH7J pattern that caused infinite loops!
        let mut source = Buffer::new(b"!!str");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
    fn test_both_decorators_on_empty() {
        let mut source = Buffer::new(b"!!str &anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();

        // Should parse as string "hello"
        assert!(matches!(result, Node::Str(s, _, _) if s == "hello"));
    }

    #[test]
    fn test_anchor_with_quoted_value() {
        let mut source = Buffer::new(b"&anchor 'hello'");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives).unwrap();

        // Should parse as alias
        assert!(matches!(result, Node::Alias(name) if name == "myalias"));
    }
}
