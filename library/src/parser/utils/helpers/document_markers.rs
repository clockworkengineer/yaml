//! Document marker helpers for YAML parsing.
//!
//! Provides classification and parsing of document start (`---`) and
//! document end (`...`) markers, including validation of inline content that
//! follows those markers.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::ISource;
use crate::parser::ParseResult;
use crate::parser::directives::DirectiveContext;
use crate::utils::{is_comment_start, is_horizontal_space, is_line_terminator};

use super::core::{parse_error_token, to_yaml_error};

/// Simple classifier for document markers in the TokenStream.
///
/// This keeps all DocumentStart / DocumentEnd detection in one place so
/// callers do not need to pattern match on the raw tokens themselves.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum DocMarkerKind {
    Start,
    End,
}

/// Classifies the current token as a document marker, if applicable.
///
/// Returns Some(DocMarkerKind::Start) when the current token is
/// Token::DocumentStart, Some(DocMarkerKind::End) for Token::DocumentEnd,
/// and None for all other tokens.
#[inline]
pub(crate) fn classify_doc_marker(
    ts: &crate::parser::token_stream::TokenStream,
) -> Option<DocMarkerKind> {
    use crate::parser::lexer::Token;
    match ts.current() {
        Some(Token::DocumentStart) => Some(DocMarkerKind::Start),
        Some(Token::DocumentEnd) => Some(DocMarkerKind::End),
        _ => None,
    }
}

/// Peeks ahead on the current line after a document start marker ("---")
/// to extract a raw tag string up to whitespace, newline, or comment.
///
/// The source position is restored before returning. Returns Some(tag)
/// when non-empty content starting at the current position is found,
/// or None when there is no tag-like content.
#[inline]
pub(crate) fn peek_tag_after_doc_start(source: &mut dyn ISource) -> Option<String> {
    let st_tag = source.save_state();
    let mut tag_raw = String::new();
    while let Some(ch) = source.current() {
        if is_horizontal_space(ch) || is_line_terminator(ch) || is_comment_start(ch) {
            break;
        }
        tag_raw.push(ch);
        source.next();
    }
    source.restore_state(st_tag);
    if tag_raw.is_empty() {
        None
    } else {
        Some(tag_raw)
    }
}

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
        let res = matches!(classify_doc_marker(&ts), Some(DocMarkerKind::Start));
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
            if is_horizontal_space(c) {
                source.next();
            } else {
                break;
            }
        }
        // Before invoking the token stream, do a character-level tag handle check
        // to reliably catch explicit handles like '!handle!Type' on the same line
        // as the document start marker, even if tokenization classifies the text
        // as Plain in some edge cases.
        if matches!(source.current(), Some(crate::constants::CHAR_EXCLAMATION)) {
            if let Some(tag_raw) = peek_tag_after_doc_start(source) {
                if let Err(e) = directives.validate_tag_handle_usage(&tag_raw) {
                    // Build a token-stream-based error for consistent formatting
                    let ts_err =
                        crate::parser::token_stream::TokenStream::new(source, directives, false)
                            .map_err(to_yaml_error)?;
                    return Err(parse_error_token(&ts_err, &e.to_string()));
                }
            }
        }
        // CXX2: Check for anchor followed by mapping on the document start line
        // This must be done at character level before creating TokenStream
        if matches!(source.current(), Some('&')) {
            let st_anchor = source.save_state();
            // Skip the anchor
            source.next(); // skip '&'
            while let Some(ch) = source.current() {
                if is_horizontal_space(ch)
                    || ch == '&'
                    || ch.is_alphanumeric()
                    || ch == '_'
                    || ch == '-'
                {
                    source.next();
                } else {
                    break;
                }
            }
            // Skip whitespace after anchor name
            while let Some(ch) = source.current() {
                if is_horizontal_space(ch) {
                    source.next();
                } else {
                    break;
                }
            }
            // Check if there's a colon on this line (indicating mapping)
            let mut has_mapping_colon = false;
            while let Some(ch) = source.current() {
                if is_line_terminator(ch) {
                    break;
                }
                if ch == ':' {
                    // Found a colon - check if it's followed by space or newline (mapping indicator)
                    source.next();
                    if let Some(next_ch) = source.current() {
                        if is_horizontal_space(next_ch) || is_line_terminator(next_ch) {
                            has_mapping_colon = true;
                        }
                    } else {
                        // EOF after colon also indicates mapping
                        has_mapping_colon = true;
                    }
                    break;
                }
                source.next();
            }
            source.restore_state(st_anchor);
            if has_mapping_colon {
                let ts_err =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                        .map_err(to_yaml_error)?;
                return Err(parse_error_token(
                    &ts_err,
                    "Mapping keys are not allowed on the same line as document start marker (---).",
                ));
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
                    Some(crate::parser::lexer::Token::Anchor(_)) => {
                        // CXX2: Anchor check is now done at character level before TokenStream creation (see above)
                        // This case should not be reached for anchors with mappings as they're caught earlier
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
        if source.current().map_or(false, is_line_terminator) {
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
            Some(c) if is_horizontal_space(c) || is_line_terminator(c) || is_comment_start(c) => {
                true
            }
            None => true,
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
                Some(c) if is_horizontal_space(c) => {
                    source.next();
                }
                Some(c) if is_comment_start(c) => {
                    // Inline comment: consume until end of line
                    while let Some(c2) = source.current() {
                        if is_line_terminator(c2) {
                            break;
                        }
                        source.next();
                    }
                }
                Some(c) if is_line_terminator(c) => break,
                None => break,
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
        } else if source.current().map_or(false, is_line_terminator) {
            source.next();
        }
    }
    Ok(consumed_end)
}
