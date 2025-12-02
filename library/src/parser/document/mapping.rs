/// Module: parser/document/mapping.rs
use crate::io::traits::ISource;
use crate::nodes::node::{BlockStyle, Node, QuoteType};
use crate::parser::document::mapping_tokens::parse_mapping_with_tokens;
use crate::parser::document::parse_value;
use crate::parser::document::sequence_tokens::parse_sequence_with_tokens;
use crate::parser::document::tokens::value::parse_value_with_tokens;

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
    /// Checks if a string value can be safely represented as plain (unquoted) YAML.
    ///
    /// Returns false if the string contains characters that require quoting
    /// or has leading/trailing spaces that would be lost in plain format.
    fn is_plain_safe_value(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        if s.starts_with(' ') || s.ends_with(' ') {
            return false;
        }
        if s.contains(['\n', '\r']) {
            return false;
        }
        let disallowed = [
            '#', '[', ']', '{', '}', '&', '*', '!', '|', '>', '"', '`', '%', '@', '\\',
        ];
        if s.chars().any(|ch| disallowed.contains(&ch)) {
            return false;
        }
        true
    }
    /// Checks if a string key can be safely represented as plain (unquoted) YAML.
    ///
    /// Similar to is_plain_safe_value but additionally excludes colons which
    /// have special meaning in YAML key-value syntax.
    fn is_plain_safe_key(s: &str) -> bool {
        is_plain_safe_value(s) && !s.contains(':')
    }

    use crate::parser::token_stream::TokenStream;

    let mut stream = TokenStream::new(source, directives)?;
    parse_mapping_with_tokens(&mut stream, _indent_level, directives)
}
