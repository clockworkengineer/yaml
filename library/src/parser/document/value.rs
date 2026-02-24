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
