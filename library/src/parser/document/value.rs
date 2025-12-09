//! Module: parser/document/value.rs

use crate::io::traits::ISource;
use crate::nodes::node::Node;

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
/// Result containing the parsed Node value or an error string
pub(crate) fn parse_value(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    // Selectively route to token-based parser for patterns that benefit:
    // - Decorators on empty values (FH7J, PW8X)
    // - Flow collections (better token boundaries)
    // - Aliases (simpler token handling)
    use crate::parser::document::bridge::{parse_value_bridged, should_use_token_parsing};

    if should_use_token_parsing(source) {
        return parse_value_bridged(source, directives);
    }

    use crate::parser::document::tokens::value::parse_value_with_tokens;
    use crate::parser::token_stream::TokenStream;

    let mut stream = TokenStream::new(source, directives, false)?;
    return parse_value_with_tokens(&mut stream, directives, 0);
}
