//! Core utility helpers for YAML document parsing.
//!
//! Provides error construction, directive handling, token matching, and
//! error formatting used throughout the rest of the helpers module.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::ParseResult;
use crate::parser::utils::error_builder::{ErrorBuilder, ErrorCategory};

/// Helper to parse and merge directives for a document.
pub(crate) fn handle_directives(
    source: &mut dyn ISource,
) -> ParseResult<crate::parser::directives::DirectiveContext> {
    let mut directives = crate::parser::directives::DirectiveContext::new();
    let parsed_directives =
        crate::parser::directives::parse_directives(source).map_err(to_yaml_error)?;
    directives.yaml_version = parsed_directives.yaml_version;
    directives
        .tag_prefixes
        .extend(parsed_directives.tag_prefixes);
    Ok(directives)
}

/// Helper to convert any error to YamlError for consistent error handling.
pub(crate) fn to_yaml_error<E: std::fmt::Display>(err: E) -> YamlError {
    YamlError::new(crate::error::ErrorKind::ParseError, format!("{}", err))
}

/// Helper to check if the current token matches a given kind.
///
/// Note: currently unused; keep for potential future TokenStream-based
/// refactors of document parsing logic.
#[allow(dead_code)]
pub(crate) fn is_token(
    ts: &crate::parser::token_stream::TokenStream,
    kind: &crate::parser::lexer::Token,
) -> bool {
    ts.current().map_or(false, |t| t == kind)
}

/// Creates a formatted error message with current token context information (TokenStream-based).
///
/// Generates an error message that includes the current token and stream position for debugging.
///
/// # Arguments
///
/// * `stream` - Reference to the TokenStream
/// * `msg` - The base error message to include
///
/// # Returns
///
/// A formatted error string with token context information
pub(crate) fn parse_error_token(
    stream: &crate::parser::token_stream::TokenStream,
    msg: &str,
) -> YamlError {
    let current = match stream.current() {
        Some(tok) => format!("{:?}", tok),
        None => "<EOF>".to_string(),
    };
    let pos = stream.stream_position();
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(&format!("{} (token: {}, pos: {})", msg, current, pos))
        .build_yaml()
}

/// Converts a node to its inline string representation for display.
///
/// Similar to the utility function but specifically tailored for parser
/// context. Provides compact string representations for debugging.
///
/// # Arguments
///
/// * `node` - A reference to the Node to convert
///
/// # Returns
///
/// A String containing the inline representation
#[allow(dead_code)]
pub(crate) fn node_to_inline_string(node: &Node) -> String {
    crate::utils::node_to_inline_string(node)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_handle_directives_parses_yaml_and_tag() {
        let yaml = b"%YAML 1.2\n%TAG !e! tag:example.com,2000:app/\n---\n";
        let mut buf = Buffer::new(yaml);
        let directives = handle_directives(&mut buf).expect("Should parse directives");
        assert_eq!(directives.yaml_version, Some((1, 2)));
        assert_eq!(
            directives.tag_prefixes.get("!e!"),
            Some(&"tag:example.com,2000:app/".to_string())
        );
    }

    #[test]
    fn test_handle_directives_duplicate_yaml_error() {
        let yaml = b"%YAML 1.2\n%YAML 1.2\n---\n";
        let mut buf = Buffer::new(yaml);
        let result = handle_directives(&mut buf);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Duplicate YAML directive"));
    }

    #[test]
    fn test_to_yaml_error_converts_string() {
        let err = to_yaml_error("custom error message");
        assert!(err.to_string().contains("custom error message"));
    }
}
