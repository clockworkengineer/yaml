/// Parses an inline YAML mapping with key-value pairs using tokens.
fn parse_inline_mapping_with_colons_tokens(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut iterations = 0;
    const MAX_PAIRS: usize = 10_000;

    loop {
        iterations += 1;
        if iterations >= MAX_PAIRS {
            return Err("Flow mapping too large or malformed - possible infinite loop".to_string());
        }

        // Check for closing brace after whitespace (handles trailing comma case)
        if let Some(Token::FlowMappingEnd) = stream.current() {
            stream.next()?;
            break;
        }

        // If we're at None (EOF) inside a flow mapping, that's an error
        if stream.current().is_none() {
            return Err("Unexpected EOF in inline mapping".to_string());
        }

        // Parse key
        use crate::parser::document::tokens::value::parse_value_with_tokens;
        let key_node = parse_value_with_tokens(stream, directives)?;

        // Expect colon
        match stream.current() {
            Some(Token::Colon) => {
                stream.next()?;
            }
            Some(tok) => return Err(format!("Expected colon in inline mapping, got: {:?}", tok)),
            None => return Err("Unexpected EOF in inline mapping (expected colon)".to_string()),
        }

        // Parse value
        let value_node = parse_value_with_tokens(stream, directives)?;

        pairs.push((key_node, value_node));

        // Skip comma or end
        match stream.current() {
            Some(Token::Comma) => {
                stream.next()?;
                // Check for trailing comma (comma followed by closing brace)
                if let Some(Token::FlowMappingEnd) = stream.current() {
                    stream.next()?;
                    break;
                }
                continue;
            }
            Some(Token::FlowMappingEnd) => {
                stream.next()?;
                break;
            }
            Some(tok) => {
                return Err(format!("Unexpected token in inline mapping: {:?}", tok));
            }
            None => return Err("Unexpected EOF in inline mapping".to_string()),
        }
    }
    Ok(Node::Mapping(pairs))
}
// Check-in: Current version of inline.rs as shown in attachments
use crate::constants::*;
use crate::error::messages::{
    ERR_EOF_INLINE_MAPPING,
    ERR_UNEXPECTED_CHAR_INLINE_MAPPING_PREFIX,
};
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::document::helpers::{
    parse_error,
};
// ...existing code...
use crate::parser::document::value::parse_value;
// ...existing code...
/// Collects a flow scalar from the source until a stop predicate is met.
/// Handles quoted scalars and line folding.
use crate::utils::skip_whitespace_and_comments_validate_tabs;
/// Collects a flow scalar from the token stream until a stop predicate is met.
/// Handles quoted scalars and line folding.
#[allow(dead_code)]
pub(crate) fn collect_flow_scalar(
    stream: &mut TokenStream,
    stop_pred: impl Fn(&Token) -> bool,
) -> String {
    let mut out = String::new();
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 10_000;

    while let Some(token) = stream.current() {
        if stop_pred(token) {
            break;
        }
        match token {
            Token::SingleQuoted(s) | Token::DoubleQuoted(s) => {
                out.push_str(s);
                let _ = stream.next();
            }
            Token::Newline => {
                let _ = stream.next();
                // Add a single space for the folded line (if we have content)
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            Token::Plain(s) => {
                out.push_str(s);
                let _ = stream.next();
            }
            _ => {
                // For other tokens, just break (could be end of scalar)
                break;
            }
        }
        iterations += 1;
        if iterations >= MAX_ITERATIONS {
            break;
        }
    }
    out
}

// ...existing code...
/// Parses an inline YAML set using tokens.
pub(crate) fn parse_inline_set(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut iterations = 0;
    const MAX_ITEMS: usize = 10_000;

    // Skip the opening '{' only if we're currently at it
    if matches!(stream.current(), Some(Token::FlowMappingStart)) {
        stream.next()?;
    }

    if let Some(Token::FlowMappingEnd) = stream.current() {
        stream.next()?;
        return Ok(Node::Mapping(pairs)); // Empty set as empty mapping
    }

    loop {
        // Prevent infinite loop
        iterations += 1;
        if iterations >= MAX_ITEMS {
            return Err("Flow set too large or malformed - possible infinite loop".to_string());
        }

        // Check for closing brace (handles trailing comma case)
        if let Some(Token::FlowMappingEnd) = stream.current() {
            stream.next()?;
            break;
        }

        // Parse the value (set item)
        use crate::parser::document::tokens::value::parse_value_with_tokens;
        let item_node = parse_value_with_tokens(stream, directives)?;

        // Add item as a key with null value (set format)
        pairs.push((item_node, Node::None));

        // Skip comma or end
        match stream.current() {
            Some(Token::Comma) => {
                stream.next()?;
                // Check for trailing comma (comma followed by closing brace)
                if let Some(Token::FlowMappingEnd) = stream.current() {
                    stream.next()?;
                    break;
                }
                continue;
            }
            Some(Token::FlowMappingEnd) => {
                stream.next()?;
                break;
            }
            Some(Token::Eof) => {
                // Gracefully end on EOF within inline set context
                break;
            }
            Some(tok) => {
                return Err(format!("Unexpected token in inline set: {:?}", tok));
            }
            None => return Err("Unexpected EOF in inline set".to_string()),
        }
    }

    Ok(Node::Mapping(pairs))
}

// Removing stray content
// hi
// O

use crate::parser::lexer::Token;
/// Parses an inline YAML mapping or set enclosed in curly braces {}.
///
/// First attempts to parse as a mapping with key-value pairs. If no colons
/// are found, parses as an inline set with comma-separated items.
/// Supports empty mappings/sets and nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing a Mapping Node or an error string
use crate::parser::token_stream::TokenStream;
/// Parses an inline YAML mapping or set using tokens.
pub(crate) fn parse_inline_mapping(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    // Look ahead to see if this is a set (no colons) or a mapping (has colons)
    let mut has_colons = false;
    let depth = 0;
    let iterations = 0;
    const MAX_LOOKAHEAD: usize = 10_000;

    stream.next()?; // Skip opening brace

    // Manual lookahead: create a temporary TokenStream from the current state
    // This requires access to the underlying source and directives
    let source_snapshot = stream.source_mut().save_state();
    let mut lookahead_stream = TokenStream::new(stream.source_mut(), directives)?;
    // Advance lookahead_stream to the same position as the original stream
    for _ in 0..iterations {
        lookahead_stream.next()?;
    }
    let mut lookahead_depth = depth;
    let mut lookahead_iterations = 0;
    while let Some(token) = lookahead_stream.current() {
        lookahead_iterations += 1;
        if lookahead_iterations >= MAX_LOOKAHEAD {
            break;
        }
        match token {
            Token::FlowMappingEnd if lookahead_depth == 0 => break,
            Token::FlowMappingStart => lookahead_depth += 1,
            Token::FlowMappingEnd => lookahead_depth -= 1,
            Token::Colon if lookahead_depth == 0 => {
                has_colons = true;
                break;
            }
            _ => {}
        }
        lookahead_stream.next()?;
    }
    // Restore the original source state
    stream.source_mut().restore_state(source_snapshot);

    if has_colons {
        parse_inline_mapping_with_colons_tokens(stream, directives)
    } else {
        parse_inline_set(stream, directives)
    }
}
/// Parses an inline YAML mapping with key-value pairs (original implementation).
#[allow(dead_code)]
fn parse_inline_mapping_with_colons(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut iterations = 0;
    const MAX_PAIRS: usize = 10_000;

    loop {
        // Prevent infinite loop
        iterations += 1;
        if iterations >= MAX_PAIRS {
            return Err(parse_error(
                source,
                "Flow mapping too large or malformed - possible infinite loop",
            ));
        }
        // Skip whitespace, newlines, and comments before parsing key
        skip_whitespace_and_comments_validate_tabs(source)?;

        // Check for closing brace after whitespace (handles trailing comma case)
        if source.current() == Some(CHAR_RBRACE) {
            source.next();
            break;
        }

        // If we're at None (EOF) inside a flow mapping, that's an error
        if source.current().is_none() {
            return Err(parse_error(source, ERR_EOF_INLINE_MAPPING));
        }

        let key_node = parse_value(source, directives)?;

        skip_whitespace_and_comments_validate_tabs(source)?;

        let value_node = parse_value(source, directives)?;

        pairs.push((key_node, value_node));

        skip_whitespace_and_comments_validate_tabs(source)?;

        match source.current() {
            Some(CHAR_COMMA) => {
                source.next();
                skip_whitespace_and_comments_validate_tabs(source)?;
                // Check if there's a closing brace after the comma (trailing comma)
                if source.current() == Some(CHAR_RBRACE) {
                    source.next();
                    break;
                }
                // Check for double comma
                if source.current() == Some(CHAR_COMMA) {
                    return Err(parse_error(source, "Flow mapping has consecutive commas"));
                }
                continue;
            }
            Some(CHAR_RBRACE) => {
                source.next();

                // Check for invalid text immediately after closing brace (no space)
                // Valid characters: whitespace, newline, comma, another closing bracket/brace, colon, or comment
                if let Some(c) = source.current() {
                    if !c.is_whitespace()
                        && c != '\n'
                        && c != '\r'
                        && c != ','
                        && c != ']'
                        && c != '}'
                        && c != '#'
                        && c != ':'
                    {
                        // Check if it's an alphanumeric character which would be clearly invalid
                        if c.is_alphanumeric() {
                            return Err(parse_error(
                                source,
                                "Invalid character after flow mapping - expected whitespace or newline",
                            ));
                        }
                    }
                }

                break;
            }
            Some(c) => {
                return Err(parse_error(
                    source,
                    &format!("{ERR_UNEXPECTED_CHAR_INLINE_MAPPING_PREFIX}{c}"),
                ));
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_MAPPING)),
        }
    }
    // After the loop, construct and return the mapping node
    Ok(Node::Mapping(pairs))
}

/// Parses an inline YAML sequence enclosed in square brackets [].
///
/// Handles comma-separated values within brackets, including nested
/// inline collections, quoted strings, and proper whitespace handling.
/// Supports empty sequences and nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing an Array Node or an error string

/// Parses an inline YAML sequence using tokens (token-based refactor).
pub(crate) fn parse_inline_sequence(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut items: Vec<Node> = Vec::new();
    let mut iterations = 0;
    const MAX_ITEMS: usize = 10_000;

    stream.next()?; // Skip the opening '['

    if let Some(Token::FlowSequenceEnd) = stream.current() {
        stream.next()?;
        return Ok(Node::Array(items));
    }

    loop {
        iterations += 1;
        if iterations >= MAX_ITEMS {
            return Err(
                "Flow sequence too large or malformed - possible infinite loop".to_string(),
            );
        }

        // Parse the value (sequence item)
        use crate::parser::document::tokens::value::parse_value_with_tokens;
        let item_node = parse_value_with_tokens(stream, directives)?;
        items.push(item_node);

        // Skip comma or end
        match stream.current() {
            Some(Token::Comma) => {
                stream.next()?;
                // Check for trailing comma (comma followed by closing bracket)
                if let Some(Token::FlowSequenceEnd) = stream.current() {
                    stream.next()?;
                    break;
                }
                continue;
            }
            Some(Token::FlowSequenceEnd) => {
                stream.next()?;
                break;
            }
            Some(Token::Newline) => {
                // Allow newlines within flow sequences, but validate indentation
                stream.skip_whitespace()?;
                continue;
            }
            Some(tok) => {
                return Err(format!("Unexpected token in inline sequence: {:?}", tok));
            }
            None => return Err("Unexpected EOF in inline sequence".to_string()),
        }
    }

    Ok(Node::Array(items))
}
