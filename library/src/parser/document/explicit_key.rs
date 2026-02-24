//! Explicit Key Parsing Helpers
//!
//! Provides helpers and utilities for parsing explicit keys in YAML mappings,
//! including token stream setup, loop guards, and normalization routines.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Helper to initialize a TokenStream and skip initial trivia for explicit key parsing.
fn setup_token_stream<'a>(
    source: &'a mut dyn ISource,
    directives: &'a crate::parser::directives::DirectiveContext,
) -> ParseResult<crate::parser::token_stream::TokenStream<'a>> {
    let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
    stream.skip_trivia()?;
    Ok(stream)
}
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::ParseResult;
use crate::parser::document::node_utils::normalize_node_to_str;
use crate::{loop_guard_check, loop_guard_init};

/// Helper for DRY loop guard usage in explicit key parsing.
fn guarded_explicit_key_loop<F>(
    mut body: F,
    max_pairs: Option<usize>,
) -> Result<Vec<(Node, Node)>, crate::error::YamlError>
where
    F: FnMut() -> Option<Result<(Node, Node), crate::error::YamlError>>,
{
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    loop_guard_init!(explicit_key_counter);
    loop {
        if let Some(max) = max_pairs {
            if pairs.len() >= max {
                break;
            }
        }
        loop_guard_check!(
            explicit_key_counter,
            crate::parser::document::loop_guards::MAX_LOOP_ITERATIONS,
            "Explicit key parsing"
        );
        match body() {
            Some(Ok((key, value))) => pairs.push((key, value)),
            Some(Err(e)) => return Err(e),
            None => break,
        }
    }
    Ok(pairs)
}

/// Core function to extract explicit key-value pairs from a TokenStream.
fn extract_explicit_key_value_pairs(
    stream: &mut crate::parser::token_stream::TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
    max_pairs: Option<usize>,
) -> Result<Vec<(Node, Node)>, crate::error::YamlError> {
    guarded_explicit_key_loop(
        || {
            stream.skip_trivia().ok()?;
            if !matches!(
                stream.current(),
                Some(crate::parser::lexer::Token::QuestionMark)
            ) {
                return None;
            }
            Some(parse_explicit_mapping_entry(stream, directives))
        },
        max_pairs,
    )
}

/// Parses multiple explicit keys and their values for mappings.
///
/// Handles the case where we have multiple consecutive lines starting with '?',
/// each followed by a value, and collects (key, value) pairs into a mapping node.

pub fn parse_multiple_explicit_keys(
    source: &mut dyn ISource,
    _indent_level: usize,
) -> crate::parser::ParseResult<Node> {
    let directives_local = crate::parser::directives::DirectiveContext::new();
    let mut stream = setup_token_stream(source, &directives_local)?;
    let pairs = extract_explicit_key_value_pairs(&mut stream, &directives_local, None)?;
    Ok(Node::Mapping(pairs))
}
// Helper functions for parsing explicit mapping keys (? indicator)

use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Checks if the current token starts an explicit key (Token::QuestionMark)
pub(crate) fn is_explicit_key_start(stream: &mut TokenStream) -> bool {
    matches!(stream.current(), Some(Token::QuestionMark))
}

/// Parses an explicit mapping key-value pair using tokens
/// Returns (key_node, value_node)
pub(crate) fn parse_explicit_mapping_entry(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> ParseResult<(Node, Node)> {
    // Check for explicit key indicator
    if !is_explicit_key_start(stream) {
        // Use stream.current() for error context since TokenStream.lexer is private
        let cur = stream.current().cloned();
        return Err(
            crate::parser::document::errors::mapping_errors::expected_explicit_key_token(
                stream, cur,
            ),
        );
    }
    stream.next()?;
    stream.skip_trivia()?;

    // Parse the key (may be empty), then normalize to string
    let mut key_node = match stream.current() {
        Some(Token::Newline) => {
            stream.next()?;
            stream.skip_trivia()?;
            // Parse document contents as key (empty explicit key)
            crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives, 0)?
        }
        _ => {
            crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives, 0)?
        }
    };
    // Normalize key_node to Node::Str if not already
    key_node = normalize_node_to_str(&key_node);

    // Skip whitespace/comments after key
    stream.skip_trivia()?;

    // Look for the colon indicator
    let value_node = match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
            stream.skip_trivia()?;
            match stream.current() {
                Some(Token::Newline) => {
                    stream.next()?;
                    stream.skip_trivia()?;
                    crate::parser::document::tokens::value::parse_value_with_tokens(
                        stream, directives, 0,
                    )?
                }
                _ => crate::parser::document::tokens::value::parse_value_with_tokens(
                    stream, directives, 0,
                )?,
            }
        }
        // No value indicator, key only
        _ => Node::None,
    };

    Ok((key_node, value_node))
}

#[allow(dead_code)]
/// Collect consecutive explicit key entries ('? key' lines) into mapping pairs.
/// Stops when the next token is not a question mark or structure changes.
pub(crate) fn collect_explicit_keys_block(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Vec<(Node, Node)>, crate::error::YamlError> {
    // Use the unified extraction function with a max_pairs limit if needed
    extract_explicit_key_value_pairs(
        stream,
        directives,
        Some(crate::parser::document::loop_guards::MAX_MAPPING_PAIRS),
    )
}
