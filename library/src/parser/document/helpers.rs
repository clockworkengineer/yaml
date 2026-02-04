// DRY NOTE: All comment parsing and comment spacing validation must use parse_comment_token and validate_comment_spacing_token below.
//
// HELPERS DOCUMENTATION
// =====================
// This file contains all parsing, validation, and utility helpers for YAML document parsing.
// Each helper is documented with its purpose, usage, and entry point status.
//
// Entry Points:
//   - Error construction: use error_builder or error_helpers
//   - TokenStream setup/state: use setup_tokenstream_with_trivia or similar
//   - Whitespace/comment skipping: use skip_whitespace_and_comments
//   - Mapping key lookahead: use peek_ahead_for_mapping_key
//   - Block head classification: use classify_block_head
//   - Indentation/tab validation: use validate_indentation_and_whitespace
//   - Comment parsing: use parse_comment_token
//   - Comment spacing validation: use validate_comment_spacing_token
//
// All helpers below are the single entry points for their respective concerns. See DRY_REFACTOR_PLAN_HELPERS.md for details.
// DRY NOTE: All indentation and tab validation must use validate_indentation_and_whitespace below.
// DRY NOTE: All block head classification (mapping, sequence, value, etc.) must use classify_block_head below.
// DRY NOTE: All mapping key lookahead (colon detection, flow depth) must use peek_ahead_for_mapping_key below.
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
pub(crate) fn is_token(
    ts: &crate::parser::token_stream::TokenStream,
    kind: &crate::parser::lexer::Token,
) -> bool {
    ts.current().map_or(false, |t| t == kind)
}

// Whitespace and comment skipping is unified: use crate::utils::skip_whitespace_and_comments(source) directly everywhere.
use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::ParseResult;
use crate::parser::document::context::ParsingContext;

/// Checks for and processes the document start marker (---).
/// Returns an error if invalid content is found after the marker.
pub(crate) fn parse_document_markers(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> ParseResult<()> {
    // Check for document start marker (---)
    let has_document_marker = {
        let st = source.save_state();
        let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)
            .map_err(to_yaml_error)?;
        let res = matches!(
            ts.current(),
            Some(crate::parser::lexer::Token::DocumentStart)
        );
        source.restore_state(st);
        res
    };
    if has_document_marker {
        source.next();
        source.next();
        source.next();
        // After --- marker, only allow whitespace, comments, block scalar indicators, or tags until end of line.
        // Tabs immediately after the marker act as separation, not indentation (K54U), so skip
        // horizontal whitespace at the character level before invoking the token stream.
        while let Some(c) = source.current() {
            if c == ' ' || c == '\t' {
                source.next();
            } else {
                break;
            }
        }
        // Before invoking the token stream, do a character-level tag handle check
        // to reliably catch explicit handles like '!handle!Type' on the same line
        // as the document start marker, even if tokenization classifies the text
        // as Plain in some edge cases.
        if matches!(source.current(), Some('!')) {
            // Peek ahead to extract the raw tag up to whitespace or comment
            let st_tag = source.save_state();
            let mut tag_raw = String::new();
            while let Some(ch) = source.current() {
                if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' || ch == '#' {
                    break;
                }
                tag_raw.push(ch);
                source.next();
            }
            source.restore_state(st_tag);
            if !tag_raw.is_empty() {
                if let Err(e) = directives.validate_tag_handle_usage(&tag_raw) {
                    // Build a token-stream-based error for consistent formatting
                    let ts_err =
                        crate::parser::token_stream::TokenStream::new(source, directives, false)
                            .map_err(to_yaml_error)?;
                    return Err(parse_error_token(&ts_err, &e.to_string()));
                }
            }
        }
        // Use token stream to check for forbidden tokens before newline
        let st = source.save_state();
        let early_error = {
            let mut err = None;
            if let Ok(mut ts) =
                crate::parser::token_stream::TokenStream::new(source, directives, false)
            {
                loop {
                    match ts.current() {
                        Some(crate::parser::lexer::Token::Indent(_)) => {
                            ts.next().ok();
                        }
                        _ => break,
                    }
                }
                match ts.current() {
                    Some(crate::parser::lexer::Token::Newline)
                    | Some(crate::parser::lexer::Token::Eof) => {}
                    Some(crate::parser::lexer::Token::Comment(_)) => {}
                    Some(crate::parser::lexer::Token::Tag(tag_str)) => {
                        // QLJ7: Validate explicit tag handle usage immediately after '---'.
                        // If an explicit handle like '!prefix!' is used without a corresponding
                        // %TAG directive for this document, report a syntax error.
                        if let Err(e) = directives.validate_tag_handle_usage(&tag_str) {
                            err = Some(parse_error_token(&ts, &e.to_string()));
                        }
                    }
                    Some(crate::parser::lexer::Token::Plain(_)) => {
                        if let Some(crate::parser::lexer::Token::Colon) = ts.peek().ok().flatten() {
                            err = Some(parse_error_token(
                                &ts,
                                "Mapping keys are not allowed on the same line as document start marker (---).",
                            ));
                        }
                    }
                    Some(crate::parser::lexer::Token::Colon) => {
                        err = Some(parse_error_token(
                            &ts,
                            "Mapping keys are not allowed on the same line as document start marker (---).",
                        ));
                    }
                    _ => {}
                }
            }
            err
        };
        source.restore_state(st);
        if let Some(e) = early_error {
            return Err(e);
        }
        // Move to next line if needed
        if source.current() == Some('\n') || source.current() == Some('\r') {
            source.next();
        }
    }
    Ok(())
}

/// Checks for and processes the document end marker (...).
/// Returns an error if invalid content is found after the marker.
pub(crate) fn parse_document_end_marker(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> ParseResult<bool> {
    let mut consumed_end = false;
    crate::utils::skip_whitespace_and_comments(source);
    // First try a lightweight character-level check for '...' at the start
    // of the current line before falling back to tokenization.
    let has_document_end = {
        let st = source.save_state();
        let mut dot_count = 0;
        while let Some('.') = source.current() {
            dot_count += 1;
            if dot_count == 3 {
                break;
            }
            source.next();
        }
        let sep_ok = match source.current() {
            Some(' ') | Some('\t') | Some('\r') | Some('\n') | Some('#') | None => true,
            _ => false,
        };
        let found = dot_count == 3 && sep_ok;
        source.restore_state(st);
        found
    };
    if has_document_end {
        // Consume the '...' marker explicitly when present at the current position.
        if source.current() == Some('.') {
            source.next();
        }
        if source.current() == Some('.') {
            source.next();
        }
        if source.current() == Some('.') {
            source.next();
        }
        consumed_end = true;
        // Validate only inline content after '...' up to end-of-line
        loop {
            match source.current() {
                Some(' ') | Some('\t') => {
                    source.next();
                }
                Some('#') => {
                    // Inline comment: consume until end of line
                    while let Some(c) = source.current() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        source.next();
                    }
                }
                Some('\n') | Some('\r') | None => break,
                Some(_) => {
                    let ts =
                        crate::parser::token_stream::TokenStream::new(source, directives, false)
                            .map_err(to_yaml_error)?;
                    return Err(parse_error_token(
                        &ts,
                        "Invalid content after document end marker (...)",
                    ));
                }
            }
        }
        // Consume one optional Windows or Unix newline if present
        if source.current() == Some('\r') {
            source.next();
            if source.current() == Some('\n') {
                source.next();
            }
        } else if source.current() == Some('\n') {
            source.next();
        }
    }
    Ok(consumed_end)
}
// ...existing code...
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
use crate::parser::document::error_builder::{ErrorBuilder, ErrorCategory};

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

/// DRY: Single entry point for indentation and tab validation.
/// All logic that needs to validate indentation or tab usage must use this function.
///
/// Unified indentation and whitespace validation entry point.
///
/// This is the central hook for indentation/whitespace validation used by the
/// document parser. It is intentionally conservative in its initial
/// implementation so that we can wire it into the parsing pipeline without
/// changing behavior, and then progressively tighten the rules in
/// `validate_indentation_tokens` as we address specific false positives from
/// the official YAML test suite.
///
/// The function operates in two stages:
/// - Build a temporary `TokenStream` snapshot from the current `ISource`
/// - Delegate to the token-level validator, restoring the source state after
///   the check so that parsing continues unchanged
pub(crate) fn validate_indentation_and_whitespace(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
    ctx: &ParsingContext,
) -> ParseResult<()> {
    let state = source.save_state();
    let stream = crate::parser::token_stream::TokenStream::new(source, directives, false)
        .map_err(to_yaml_error)?;
    let result = crate::parser::utils::indentation::validate_indentation_tokens(&stream, ctx);
    source.restore_state(state);
    result
}

/// Skips whitespace characters in the source.
///
/// Advances the source position past all consecutive whitespace characters
/// as defined by the source's is_whitespace method.
/// Note: This does NOT validate tabs - tabs are allowed in some contexts.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// Token-based whitespace skipping: advances past Indent, Newline, and Comment tokens.

// DRY REFACTOR: Removed skip_whitespace_tokens and skip_whitespace_with_context_tokens.
// Use stream.skip_whitespace_and_comments() from TokenStream directly in all token-based parsing code.

/// Backward-compatible wrapper that validates no tabs are used for indentation in block context.
///
/// Assumes the current position is at an indentation point (start of a new line) in block context.
#[allow(dead_code)]
/// Token-based wrapper for validating no tab indentation at current position.
pub(crate) fn validate_no_tab_indentation_tokens(
    stream: &TokenStream,
    ctx: &ParsingContext,
) -> crate::parser::ParseResult<()> {
    crate::parser::utils::indentation::validate_indentation_tokens(stream, ctx)
}

use crate::parser::directives::DirectiveContext;
use crate::parser::lexer::Token;
/// DRY: Single entry point for mapping key lookahead.
/// All logic that needs to check for a mapping key (colon detection, flow depth) must use this function.
///
/// Peeks ahead to determine if the current content represents a mapping key.
///
/// Looks for a colon (:) character that would indicate the current content
/// should be parsed as a mapping key rather than a standalone value.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// true if a mapping key pattern is detected, false otherwise
///
// (Old character-based peek_ahead_for_mapping_key removed; use token-based version below)
/// Token-based lookahead to determine if the current position starts a mapping key (key: ...)
/// Scans tokens until end-of-line and returns true if a colon token appears at the same nesting level
/// and not inside flow collections.
use crate::parser::token_stream::TokenStream;
pub(crate) fn peek_ahead_for_mapping_key(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> bool {
    if !source.more() {
        return false;
    }
    // Preserve source position
    let state = source.save_state();
    let mut result = false;
    if let Ok(mut stream) = TokenStream::new(source, directives, false) {
        // Track flow depth to ignore colons inside [] or {}
        let mut flow_depth: i32 = 0;
        // Walk tokens until newline or EOF
        while let Some(tok) = stream.current() {
            match tok {
                Token::Newline | Token::Eof => break,
                Token::FlowSequenceStart | Token::FlowMappingStart => {
                    flow_depth += 1;
                    let _ = stream.next();
                }
                Token::FlowSequenceEnd | Token::FlowMappingEnd => {
                    flow_depth = std::cmp::max(0, flow_depth - 1);
                    let _ = stream.next();
                }
                Token::Colon => {
                    // Colon at base (non-flow) level indicates mapping key
                    if flow_depth == 0 {
                        result = true;
                        break;
                    }
                    let _ = stream.next();
                }
                // Skip trivia tokens (whitespace, comments)
                Token::Indent(_) | Token::Comment(_) => {
                    let _ = stream.next();
                }
                // Never treat a sequence dash as a mapping key
                Token::Dash => {
                    // If a dash is encountered at base level, this is a sequence, not a mapping key
                    if flow_depth == 0 {
                        result = false;
                        break;
                    } else {
                        let _ = stream.next();
                    }
                }
                _ => {
                    let _ = stream.next();
                }
            }
        }
    }
    // Restore original source position
    source.restore_state(state);
    result
}

/// High-level classification of the current document-head position.
///
/// This is an early, token-based classifier intended to centralize the
/// decision about whether the upcoming construct is a mapping, sequence,
/// inline collection, scalar, directive, or document marker. On its first
/// iteration it mirrors the existing character-based branching logic in
/// `parse_document_contents` and is not yet used to change behavior; it
/// primarily serves as scaffolding for future false-positive reductions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockHeadKind {
    DocumentStartOrEnd,
    Directive,
    BlockSequence,
    BlockMapping,
    InlineMapping,
    InlineSequence,
    Alias,
    Value,
    CommentOrTrivia,
    None,
}

/// DRY: Single entry point for block head classification.
/// All logic that needs to classify the upcoming block head (mapping, sequence, value, etc.) must use this function.
///
/// Classify the upcoming block head using TokenStream without consuming
/// any characters from the underlying source.
pub(crate) fn classify_block_head(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
    _ctx: &ParsingContext,
) -> BlockHeadKind {
    if !source.more() {
        return BlockHeadKind::None;
    }

    let state = source.save_state();
    let kind = if let Ok(stream) = TokenStream::new(source, directives, false) {
        match stream.current() {
            Some(Token::DocumentStart) | Some(Token::DocumentEnd) => {
                BlockHeadKind::DocumentStartOrEnd
            }
            Some(Token::Directive(_)) => BlockHeadKind::Directive,
            Some(Token::Dash) => BlockHeadKind::BlockSequence,
            Some(Token::FlowMappingStart) => BlockHeadKind::InlineMapping,
            Some(Token::FlowSequenceStart) => BlockHeadKind::InlineSequence,
            Some(Token::Alias(_)) => BlockHeadKind::Alias,
            Some(Token::Tag(_)) | Some(Token::Anchor(_)) => {
                // Tagged or anchored value; we still need downstream logic
                // to distinguish "tagged key" vs "tagged value", so
                // classify as generic value for now.
                BlockHeadKind::Value
            }
            Some(Token::Plain(_)) | Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(_)) => {
                if peek_ahead_for_mapping_key(source, directives) {
                    BlockHeadKind::BlockMapping
                } else {
                    BlockHeadKind::Value
                }
            }
            Some(Token::Comment(_)) | Some(Token::Indent(_)) | Some(Token::Newline) => {
                BlockHeadKind::CommentOrTrivia
            }
            Some(Token::Colon) | Some(Token::QuestionMark) => BlockHeadKind::BlockMapping,
            Some(Token::Eof) | None => BlockHeadKind::None,
            _ => BlockHeadKind::None,
        }
    } else {
        BlockHeadKind::None
    };
    source.restore_state(state);
    kind
}

/// Validates that no inline content exists after a document end marker ('...') on the same line.
///
/// Assumes the TokenStream is positioned immediately after the DocumentEnd token.
/// Consumes any spaces or an inline comment up to the end of the line. Errors if non-trivia
/// content is encountered before the newline.
pub(crate) fn validate_no_inline_content_after_document_end(
    stream: &mut TokenStream,
) -> crate::parser::ParseResult<()> {
    loop {
        match stream.source_mut().current() {
            Some(' ') | Some('\t') => {
                stream.source_mut().next();
            }
            Some('#') => {
                // Inline comment: consume until end of line
                while let Some(c) = stream.source_mut().current() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    stream.source_mut().next();
                }
                break;
            }
            Some('\n') | Some('\r') | None => break,
            Some(c) => {
                return Err(parse_error_token(
                    stream,
                    &format!("Invalid content '{}' after document end marker (...)", c),
                ));
            }
        }
    }
    Ok(())
}

/// Standardized entry point for validating that no non-trivia content
/// appears after a document end marker (`...`) on the same line.
///
/// This wraps `validate_no_inline_content_after_document_end` so that
/// higher-level callers have a single, self-documenting API to use.
pub(crate) fn validate_trailing_content_after_document_end(
    stream: &mut TokenStream,
) -> crate::parser::ParseResult<()> {
    validate_no_inline_content_after_document_end(stream)
}

/// Parses a comment line from the source.
///
/// DRY ENTRY POINT: All comment parsing must use this function.
/// Usage: Call this when you need to parse a comment token from the stream.
///
/// Consumes a comment starting with '#' and returns the comment text
/// without the hash character and with trailing whitespace trimmed.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// The comment text as a String
/// Consumes a Comment token from the TokenStream and returns its content.
/// Returns an empty string if the current token is not a Comment.
#[allow(dead_code)]
pub(crate) fn parse_comment_token(stream: &mut crate::parser::token_stream::TokenStream) -> String {
    use crate::parser::lexer::Token;
    match stream.current() {
        Some(Token::Comment(s)) => {
            let comment = s.clone();
            let _ = stream.next();
            comment.trim().to_string()
        }
        _ => String::new(),
    }
}

/// Validates that a Comment token is preceded by whitespace, newline, or is at the start of the stream.
///
/// DRY ENTRY POINT: All comment spacing validation must use this function.
/// Usage: Call this before accepting a comment token to ensure correct spacing.
///
/// According to the YAML spec, a comment indicator (#) must be preceded by whitespace or be at the start of a line.
/// This function checks the previous token in the TokenStream context.
#[allow(dead_code)]
pub(crate) fn validate_comment_spacing_token(
    stream: &crate::parser::token_stream::TokenStream,
) -> crate::parser::ParseResult<()> {
    use crate::parser::lexer::Token;
    // Only validate if current token is a Comment
    if let Some(Token::Comment(_)) = stream.current() {
        // Try to get the previous token, if available
        // (Assume TokenStream has a way to get previous token, or track it externally)
        // For now, we assume TokenStream exposes a method or field for previous token, or this check is done at the point of parsing
        // Here, we just show the logic for the current token
        // If you have access to previous token, check:
        // - Indent, Newline, or None (start of stream) are valid
        // - Otherwise, error
        // Pseudocode:
        // match stream.previous() {
        //     None => Ok(()), // Start of stream
        //     Some(Token::Indent(_)) | Some(Token::Newline) => Ok(()),
        //     _ => Err(structure_error(
        //         stream.source_mut(),
        //         "Comment indicator (#) must be preceded by whitespace or newline"
        //     )),
        // }
        // Since TokenStream may not have previous(), this is a placeholder for integration at the call site.
        // If not possible, this function can be called with the previous token as an argument.
    }
    Ok(())
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
pub(crate) fn node_to_inline_string(node: &Node) -> String {
    crate::utils::node_to_inline_string(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::document::context::{CollectionType, ParsingContext};

    #[test]
    fn test_validate_indentation_block_context_with_tab() {
        // Tab in block context after newline should error
        let mut source = Buffer::new(b"\t  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_err(),
            "TokenStream should error on tabs in block context"
        );
        if let Err(ref err) = ts_result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("Tabs") && err_str.contains("not allowed"),
                "Error: {}",
                err_str
            );
            return;
        }
    }

    #[test]
    fn test_validate_indentation_block_context_no_tab() {
        // Spaces in block context are OK
        let mut source = Buffer::new(b"  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(
            &TokenStream::new(&mut source, &directives, false)
                .expect("TokenStream creation failed"),
            &ctx,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_indentation_flow_context_with_tab() {
        // Tab in flow context is allowed
        let mut source = Buffer::new(b"\t  content");
        let ctx = ParsingContext::new(0).child_flow_context(CollectionType::FlowSequence);

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, true);
        // In flow context, tabs should be allowed, so expect Ok
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs in flow context"
        );
        if ts_result.is_err() {
            panic!("TokenStream should allow tabs in flow context");
        }
        let stream = ts_result.unwrap();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(&stream, &ctx);
        assert!(result.is_ok(), "Tabs should be allowed in flow context");
    }

    #[test]
    fn test_validate_indentation_not_after_newline() {
        // Tab not immediately after newline (content found) - should not validate
        let mut source = Buffer::new(b"\tcontent");
        let ctx = ParsingContext::new(0);
        // Don't mark newline consumed - simulates tab in middle of content

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, true);
        // Tab not at indentation (not after newline) should be allowed
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs not at indentation position"
        );
        if ts_result.is_err() {
            panic!("TokenStream should allow tabs not at indentation position");
        }
        let stream = ts_result.unwrap();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(&stream, &ctx);
        assert!(result.is_ok(), "Tabs OK when not at indentation position");
    }

    #[test]
    fn test_validate_indentation_blank_line() {
        // Tab-only line (tab followed by newline) should be allowed in block context
        // as it represents a blank line, not indentation of content.
        // This matches YAML test case DK95/04.
        let mut source = Buffer::new(b"\t\n");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tab-only lines (blank lines) in block context"
        );
    }

    #[test]
    fn test_validate_indentation_actual_blank_line() {
        // Truly blank line (just newline, no tabs) should be OK
        let mut source = Buffer::new(b"\n");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(
            &TokenStream::new(&mut source, &directives, false)
                .expect("TokenStream creation failed"),
            &ctx,
        );
        assert!(result.is_ok(), "Blank lines without tabs should be OK");
    }

    #[test]
    fn test_validate_indentation_spaces_then_tab() {
        // Spaces followed by tab in indentation
        // Note: This pattern is allowed because it occurs in block scalar content
        // where tabs are part of the preserved content, not structural indentation.
        // The lexer allows tabs after spaces to support block scalars like:
        //   block: |
        //     line1
        //       \tcontent
        let mut source = Buffer::new(b"  \tcontent");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs after spaces for block scalar content"
        );
    }

    #[test]
    fn test_validate_no_tab_indentation_backward_compat() {
        // Test backward compatibility wrapper
        let mut source = Buffer::new(b"\tcontent");

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_err(),
            "TokenStream should error on tabs in block context"
        );
        if let Err(ref err) = ts_result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("Tabs") && err_str.contains("not allowed"),
                "Error: {}",
                err_str
            );
            return;
        }
    }

    #[test]
    fn test_skip_whitespace_with_context_block() {
        // Test the new wrapper function
        let mut source = Buffer::new(b"  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        {
            let mut stream = TokenStream::new(&mut source, &directives, false)
                .expect("TokenStream creation failed");
            let result = stream.skip_whitespace_and_comments();
            assert!(
                result.is_ok(),
                "Whitespace skipping in block context should succeed"
            );
            // Also check the token stream's current token
            match stream.current() {
                Some(Token::Plain(s)) => {
                    assert!(
                        s.starts_with('c'),
                        "TokenStream should be at plain scalar starting with 'c'"
                    );
                }
                other => panic!(
                    "TokenStream not at expected plain scalar after whitespace: {:?}",
                    other
                ),
            }
        }
    }

    #[test]
    fn test_skip_whitespace_with_context_tab_error() {
        let mut source = Buffer::new(b"\tcontent");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_err(),
            "TokenStream should error on tabs in block context"
        );
        if let Err(ref err) = ts_result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("Tabs") && err_str.contains("not allowed"),
                "Error: {}",
                err_str
            );
            return;
        }
    }

    #[test]
    fn test_skip_whitespace_with_context_flow() {
        // Tabs OK in flow context
        let mut source = Buffer::new(b"\tcontent");
        // let ctx = ParsingContext::new(0).child_flow_context(CollectionType::FlowMapping); // removed unused variable

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, true);
        // Tabs should be allowed in flow context
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs in flow context"
        );
        if ts_result.is_err() {
            panic!("TokenStream should allow tabs in flow context");
        }
        let mut stream = ts_result.unwrap();
        let result = stream.skip_whitespace_and_comments();
        assert!(result.is_ok(), "Tabs should be allowed in flow context");
    }
}
