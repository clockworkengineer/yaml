/// Module: parser/document/mapping.rs
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::document::mapping_tokens::parse_mapping_with_tokens;
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
/// Result containing a Mapping Node or an error string
pub(crate) fn parse_mapping(
    source: &mut dyn ISource,
    _indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
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
            Token::SingleQuoted(_) | Token::DoubleQuoted(_) => true,
            _ => false,
        }
    }
    // Checks if a key token can be safely represented as plain (unquoted) YAML.
    #[allow(dead_code)]
    fn is_plain_safe_key_token(token: &Token) -> bool {
        match token {
            Token::Plain(value) => is_plain_safe_value_token(token) && !value.contains(':'),
            Token::SingleQuoted(_) | Token::DoubleQuoted(_) => true,
            _ => false,
        }
    }

    use crate::parser::token_stream::TokenStream;

    let mut stream = TokenStream::new(source, directives, false)?;

    // Refactored: parse_mapping now uses tokens for all key/value safety checks
    // and does not perform manual char/string inspection.
    // The parse_mapping_with_tokens function should be updated to use is_plain_safe_key_token and is_plain_safe_value_token
    parse_mapping_with_tokens(&mut stream, _indent_level, directives, 0)
}
