//! Peek-ahead and block-head classification helpers for YAML parsing.
//!
//! Provides lookahead to distinguish mapping keys from plain scalars, and
//! classification of the upcoming block construct type without consuming any
//! source characters.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::ISource;
use crate::parser::directives::DirectiveContext;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
use crate::parser::utils::context::ParsingContext;

/// DRY: Single entry point for mapping key lookahead.
/// All logic that needs to check for a mapping key (colon detection, flow depth) must use this function.
///
/// Peeks ahead to determine if the current content represents a mapping key.
///
/// Looks for a colon (:) character that would indicate the current content
/// should be parsed as a mapping key rather than a standalone value.
///
/// # Returns
///
/// true if a mapping key pattern is detected, false otherwise
///
/// Token-based lookahead to determine if the current position starts a mapping key (key: ...)
/// Scans tokens until end-of-line and returns true if a colon token appears at the same nesting level
/// and not inside flow collections.
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
            Some(Token::Plain(_))
            | Some(Token::SingleQuoted(_))
            | Some(Token::DoubleQuoted(..)) => {
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
