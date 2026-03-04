//! Document Contents Parsing
//!
//! Implements parsing logic for YAML document contents, including error construction macros,
//! indentation validation, and handling of explicit keys, block heads, and document markers.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::parser::tokens::mapping::parse_mapping_with_tokens;

/// Macro for common error construction
macro_rules! parse_err {
    ($msg:expr) => {
        YamlError::from($msg)
    };
    ($stream:expr, $msg:expr) => {
        YamlError::from(helpers::parse_error_token($stream, $msg))
    };
}
// Indentation validation is centralized in indentation.rs to keep policy changes in one place.
use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::ParseResult;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::explicit_key::parse_multiple_explicit_keys;
use crate::parser::document::indentation::{
    ensure_indent_at_least, ensure_indent_at_least_no_source,
};
use crate::parser::document::mapping::parse_mapping;
use crate::parser::document::value::parse_value;
use crate::parser::utils::context::{CollectionType, ParsingContext};
use crate::parser::utils::helpers;
use crate::parser::utils::helpers::{BlockHeadKind, DocMarkerKind, classify_doc_marker};

/// Unified entry point for scalar/value parsing
fn parse_scalar_or_value(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
    indent_level: usize,
    ctx: &ParsingContext,
) -> ParseResult<Node> {
    // Top-level plain multiline scalar (HS5T)
    if matches!(ctx.collection_type, CollectionType::None)
        && matches!(source.current(), Some(c) if c.is_alphanumeric())
    {
        parse_plain_multiline_scalar(source, indent_level)
    } else {
        parse_value(source, directives)
    }
}

/// Parse a top-level plain scalar spanning multiple non-empty lines, using
/// YAML's plain line folding rules (spec example 7.12 / HS5T).
///
/// Returns true if `line` (already trimmed) looks like a YAML mapping entry,
/// i.e. it contains `": "` or ends with `':'`. Per YAML spec §7.3.3, such
/// patterns are forbidden inside a plain scalar block because they are
/// interpreted as mapping value indicators (2CMS).
fn line_looks_like_mapping_entry(line: &str) -> bool {
    line.contains(": ") || line.ends_with(':')
}

/// Treats consecutive non-empty lines as a "paragraph" where lines are
/// folded into a single space, and blank lines separate paragraphs which
/// are joined with a newline. Leading and trailing whitespace on each line
/// (including tabs) is trimmed before folding, so indentation is preserved
/// only structurally through folding, not as significant indentation.
///
/// Returns an error if:
/// - The first line does NOT look like a mapping key (i.e., is a plain scalar)
/// - AND a continuation line (any line after the first) contains a mapping-value
///   indicator (`": "` or trailing `:`), which is forbidden in plain block scalars
///   (YAML test suite case 2CMS).
///
/// When the first line itself looks like a mapping key, the function skips the
/// continuation check — those cases are misrouted mappings where the check would
/// produce false positives.
fn parse_plain_multiline_scalar(source: &mut dyn ISource, base_indent: usize) -> ParseResult<Node> {
    let mut paragraphs: Vec<Vec<String>> = Vec::new();
    let mut current_paragraph: Vec<String> = Vec::new();
    let mut is_first_line = true;
    // Track whether the first line looks like a plain scalar (not a mapping key).
    // Only when it is a plain scalar do we enforce the continuation-line check.
    let mut first_line_is_plain_scalar = false;
    // Remember the indent (column position) at the start of the first line.
    // In the normal case (called at the very beginning of a line), this will be 0.
    // In misrouted cases (function accidentally called mid-line, e.g. after a
    // TokenStream exits leaving the source mid-way through `- 2`), this will be
    // a non-zero column.  Continuation lines always start fresh at column 0.
    // By requiring `line_indent >= first_line_indent` we skip the check for
    // lines that are clearly at a different structural level (2CMS vs AZ63).
    let mut first_line_indent: usize = 0;

    loop {
        if !source.more() {
            break;
        }

        // Save the start of this line so we can restore if we encounter
        // a document marker ("---" or "...") at the base indentation
        // level. This avoids folding document markers into the scalar
        // content (which would break round-trip expectations) while still
        // allowing HS5T-style plain multi-line scalars without markers.
        let line_start_state = source.save_state();
        let line_indent = source.get_current_indent_level();

        // Detect whether this line carries an inline comment (` #` or `\t#`).
        // Per YAML spec §7.3.3, a comment terminates a plain scalar.  When a
        // line in a plain scalar block has an inline comment, the continuation
        // stops here — the next line is a separate value, not a continuation.
        let had_inline_comment = {
            let st = source.save_state();
            let raw_s = crate::utils::collect_until(source, |c| c == '\n' || c == '\r');
            let has_comment = raw_s.contains(" #") || raw_s.contains("\t#");
            source.restore_state(st);
            has_comment
        };

        // Read current line content, trimmed and without inline comments.
        let line = crate::utils::read_line_trimmed_into_string(source);

        // If this trimmed line is a document start/end marker at the
        // same indentation as the scalar's base, stop before consuming
        // it so the main document parser can handle it as a marker.
        if (line == "..." || line == "---") && line_indent == base_indent {
            source.restore_state(line_start_state);
            break;
        }

        if is_first_line && !line.is_empty() {
            // The first line is a plain scalar if it looks nothing like a
            // mapping key (no ": " marker and does not end with ":").
            first_line_is_plain_scalar = !line_looks_like_mapping_entry(&line);
            first_line_indent = line_indent;
        }

        // Per YAML spec §7.3.3, a plain scalar in block context cannot span
        // a continuation line that itself looks like a mapping entry (i.e.
        // contains `": "` or ends with `:`). Such input is inherently
        // ambiguous and must be rejected (2CMS).
        //
        // The check is guarded by two conditions to avoid false positives:
        //
        // 1. `first_line_is_plain_scalar`: only fire when the first line was
        //    itself a plain scalar (not a mapping key).  Inputs whose first
        //    line looks like `key:` are misrouted mappings; skip the check.
        //
        // 2. `line_indent >= first_line_indent`: in misrouted cases this
        //    function is called mid-line (column > 0), so `first_line_indent`
        //    is non-zero.  Continuation lines always start fresh at column 0,
        //    so `line_indent (= 0) < first_line_indent (> 0)` is satisfied.
        //    In the correct 2CMS case both values are 0 (called at line start),
        //    so the guard passes and the check applies.
        if !is_first_line
            && !line.is_empty()
            && first_line_is_plain_scalar
            && line_indent >= first_line_indent
            && line_looks_like_mapping_entry(&line)
        {
            return Err(crate::error::YamlError::from(
                "Invalid mapping entry in plain multiline scalar: \
                continuation line contains a mapping value indicator (\": \") \
                which is forbidden in plain block scalars",
            ));
        }

        // Consume the newline (LF or CRLF) if present.
        match source.current() {
            Some('\r') => {
                source.next();
                if matches!(source.current(), Some('\n')) {
                    source.next();
                }
            }
            Some('\n') => {
                source.next();
            }
            _ => {}
        }

        if line.is_empty() {
            if !current_paragraph.is_empty() {
                paragraphs.push(current_paragraph);
                current_paragraph = Vec::new();
            }
        } else {
            current_paragraph.push(line.clone());
        }

        is_first_line = false;

        // BS4K: if this line had an inline comment the plain scalar ends here.
        // Do not fold subsequent lines into this scalar; the next line is a
        // separate top-level value (or another key/value in the outer context).
        if had_inline_comment && !line.is_empty() {
            break;
        }
    }

    if !current_paragraph.is_empty() {
        paragraphs.push(current_paragraph);
    }

    // Fold lines inside each paragraph with spaces, and join paragraphs
    // with newlines to match HS5T expectations.
    let mut parts: Vec<String> = Vec::new();
    for para in paragraphs {
        parts.push(para.join(" "));
    }
    let combined = parts.join("\n");

    Ok(Node::Str(
        combined,
        crate::nodes::node::QuoteType::Unquoted,
        crate::nodes::node::BlockStyle::None,
    ))
}

/// Fast token-dispatch for block constructs to prefer tokenized paths.
fn token_dispatch(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
) -> Option<ParseResult<Node>> {
    let st = source.save_state();
    if let Ok(ts) = crate::parser::token_stream::TokenStream::new(source, directives, false) {
        match ts.current() {
            Some(crate::parser::lexer::Token::Indent(lvl)) => {
                let level_val = *lvl;
                source.restore_state(st);
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                        .ok()?;
                let result = parse_mapping_with_tokens(&mut stream, level_val, directives, 0);
                return Some(result.map_err(YamlError::from));
            }
            Some(crate::parser::lexer::Token::Dash) => {
                // Only treat dash as sequence start if not in flow context or explicit key context
                if ctx.in_flow || matches!(ctx.collection_type, CollectionType::BlockMapping) {
                    source.restore_state(st);
                    return None;
                }
                source.restore_state(st);
                let seq_indent = source.get_current_indent_level();
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                        .ok()?;
                let ctx_seq = ctx.child_block_context(seq_indent, CollectionType::BlockSequence);
                let result = crate::parser::tokens::sequence::parse_sequence_with_tokens(
                    &mut stream,
                    seq_indent,
                    ctx.indent_level,
                    directives,
                    &ctx_seq,
                    0,
                );
                return Some(result.map_err(YamlError::from));
            }
            _ => {
                source.restore_state(st);
            }
        }
    } else {
        source.restore_state(st);
    }
    None
}

/// Checks if the current position is at a document end marker (...).
fn is_doc_end(source: &mut dyn ISource, directives: &DirectiveContext) -> ParseResult<bool> {
    let st = source.save_state();
    let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
    let res = matches!(classify_doc_marker(&ts), Some(DocMarkerKind::End));
    source.restore_state(st);
    Ok(res)
}

/// Handles multiple explicit keys at the same indentation level.
fn handle_multiple_explicit_keys(
    source: &mut dyn ISource,
    current_indent: usize,
) -> ParseResult<Node> {
    // Now collects (key, value) pairs for all explicit keys at this indent
    Ok(parse_multiple_explicit_keys(source, current_indent).map_err(YamlError::from)?)
}

/// Helper to skip whitespace and comments with context-aware tab validation.
/// In block context, validates that tabs are not used for indentation after newlines.
fn skip_trivia_with_ctx(
    source: &mut dyn ISource,
    ctx: &ParsingContext,
) -> crate::parser::ParseResult<()> {
    if !ctx.in_flow {
        match crate::utils::skip_whitespace_and_comments_validate_tabs(source) {
            Ok(()) => Ok(()),
            Err(_e) => Err(crate::parser::errors::indentation_errors::IndentationErrors::tabs_not_allowed_yaml_block(source)),
        }
    } else {
        crate::utils::skip_whitespace_and_comments(source);
        Ok(())
    }
}

/// Parses the contents of a YAML document based on the current character and context.
///
/// Determines the appropriate parsing strategy based on the current character:
/// sequences (-), comments (#), inline mappings ({}), inline sequences ([]),
/// explicit mapping keys (?), block scalars (| or >), or regular mappings.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The indentation level for proper nesting
/// * `directives` - Directive context for tag resolution and version-specific parsing
///
/// # Returns
///
/// Result containing a Node or an error string
pub fn parse_document_contents(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
) -> ParseResult<Node> {
    skip_trivia_with_ctx(source, ctx)?;
    crate::parser::utils::helpers::validate_indentation_and_whitespace(source, directives, ctx)?;
    let head_kind = helpers::classify_block_head(source, directives, ctx);

    // Handle explicit keys first so token-dispatch doesn't overshadow '?'
    if matches!(source.current(), Some('?')) {
        let current_indent = source.get_current_indent_level();
        return handle_multiple_explicit_keys(source, current_indent);
    }

    // Prefer token-dispatch for mappings/sequences when classifier
    // indicates such constructs; otherwise fall back to the existing
    // logic. This keeps current behavior while centralizing the
    // decision about when to try token-based parsing.
    if matches!(
        head_kind,
        BlockHeadKind::BlockMapping | BlockHeadKind::BlockSequence
    ) {
        if let Some(result) = token_dispatch(source, directives, ctx) {
            return result;
        }
    }

    match source.current() {
        Some(c) if c == crate::constants::CHAR_DASH => {
            if ctx.in_flow || matches!(ctx.collection_type, CollectionType::BlockMapping) {
                return parse_scalar_or_value(source, directives, indent_level, ctx);
            }
            let seq_indent = source.get_current_indent_level();
            // Validate indentation without borrowing source to avoid conflicts with TokenStream
            ensure_indent_at_least_no_source(seq_indent, indent_level, "Sequence item")?;
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::DocumentStart) => {
                    return Ok(Node::None);
                }
                Some(crate::parser::lexer::Token::Dash) => {
                    let ctx_seq =
                        ctx.child_block_context(seq_indent, CollectionType::BlockSequence);
                    let seq = crate::parser::tokens::sequence::parse_sequence_with_tokens(
                        &mut stream,
                        seq_indent,
                        indent_level,
                        directives,
                        &ctx_seq,
                        0,
                    )?;
                    if let Node::None = seq {
                        return Ok(Node::None);
                    }
                    Ok(seq)
                }
                _ => parse_scalar_or_value(source, directives, indent_level, ctx),
            }
        }
        Some(c) if c == crate::constants::CHAR_DOT => {
            let map_indent = source.get_current_indent_level();
            if is_doc_end(source, directives)? {
                return Ok(Node::None);
            }
            ensure_indent_at_least(source, map_indent, indent_level, "Mapping key")?;
            if matches!(head_kind, BlockHeadKind::BlockMapping) {
                Ok(parse_mapping(source, map_indent, directives)?)
            } else {
                parse_scalar_or_value(source, directives, indent_level, ctx)
            }
        }
        Some(c) if c == crate::constants::CHAR_HASH => {
            skip_trivia_with_ctx(source, ctx)?;
            parse_document_contents(source, indent_level, directives, ctx)
        }
        Some(c) if c == crate::constants::CHAR_LBRACE => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, true)?;
            Ok(
                crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
                    &mut stream,
                    directives,
                    0,
                    false,
                    None,
                )?,
            )
        }
        Some(c) if c == crate::constants::CHAR_LBRACKET => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, true)?;
            let start_line = stream.source_line();
            let result = crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens(
                &mut stream,
                directives,
                0,
                None,
            )?;
            // C2SP: A flow sequence spanning multiple source lines used as an
            // implicit block mapping key is invalid (YAML spec §8.1.1).
            // After parsing, if the sequence consumed a newline AND a bare ':'
            // follows (implicit-key indicator on the block level), error.
            if stream.source_line() > start_line {
                if matches!(stream.current(), Some(crate::parser::lexer::Token::Colon)) {
                    return Err(
                        "Parse error: implicit block mapping key spans multiple lines \
                         (multiline flow collection cannot be an implicit block mapping key)"
                            .to_string()
                            .into(),
                    );
                }
            }
            Ok(result)
        }
        Some(c) if c == crate::constants::CHAR_EXCLAMATION => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::Tag(_)) => {
                    // Do not consume the tag here; allow value parser to
                    // handle decorators (tag/anchor) and enforce validation.
                    // Determine if this represents a mapping key by peeking
                    // ahead for a colon after the tag.
                    let is_mapping_key = match stream.peek() {
                        Ok(Some(crate::parser::lexer::Token::Colon)) => true,
                        _ => false,
                    };
                    if is_mapping_key {
                        Ok(parse_mapping(source, indent_level, directives)?)
                    } else {
                        Ok(crate::parser::tokens::value::parse_value_with_tokens(
                            &mut stream,
                            directives,
                            0,
                        )?)
                    }
                }
                _ => Ok(crate::parser::tokens::value::parse_value_with_tokens(
                    &mut stream,
                    directives,
                    0,
                )?),
            }
        }
        Some(c) if c == '&' => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::Anchor(_)) => {
                    stream.next()?;
                    // After '&anchor', decide next action based on immediate token,
                    // without skipping trivia so we can distinguish same-line vs newline.
                    match stream.current() {
                        Some(crate::parser::lexer::Token::Colon) => {
                            Ok(parse_mapping(source, indent_level, directives)?)
                        }
                        Some(crate::parser::lexer::Token::Dash) => {
                            let mut ts_err = crate::parser::token_stream::TokenStream::new(
                                source, directives, false,
                            )?;
                            Err(crate::parser::errors::anchor_errors::AnchorErrors::anchor_cannot_precede_dash_same_line(&mut ts_err))
                        }
                        Some(crate::parser::lexer::Token::Plain(s)) => {
                            if s.trim_start().starts_with('-') {
                                let mut ts_err = crate::parser::token_stream::TokenStream::new(
                                    source, directives, false,
                                )?;
                                Err(crate::parser::errors::anchor_errors::AnchorErrors::anchor_cannot_precede_dash_same_line(&mut ts_err))
                            } else {
                                Ok(crate::parser::tokens::value::parse_value_with_tokens(
                                    &mut stream,
                                    directives,
                                    0,
                                )?)
                            }
                        }
                        // Newline or anything else: defer to value parser
                        _ => Ok(crate::parser::tokens::value::parse_value_with_tokens(
                            &mut stream,
                            directives,
                            0,
                        )?),
                    }
                }
                _ => Ok(crate::parser::tokens::value::parse_value_with_tokens(
                    &mut stream,
                    directives,
                    0,
                )?),
            }
        }
        Some(c) if c == '*' => parse_scalar_or_value(source, directives, indent_level, ctx),
        Some(c) if c == ':' => {
            let current_indent = source.get_current_indent_level();
            let mut pairs: Vec<(Node, Node)> = Vec::new();
            loop {
                if source.get_current_indent_level() != current_indent
                    || source.current() != Some(':')
                {
                    break;
                }
                source.next();
                if source.current() == Some('\t') {
                    let stream =
                        crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                    return Err(parse_err!(
                        &stream,
                        "Tabs cannot be used as separation after explicit value indicator"
                    ));
                }
                crate::utils::skip_whitespace_and_comments(source);

                let value_node = {
                    // Contain `stream` inside this block so it is dropped before
                    // we need to inspect `source` directly for the QLJ7 doc-marker
                    // check below. The stream borrows `source` mutably and must be
                    // released before we can call `source.save_state()`.
                    let mut stream =
                        crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                    match stream.current() {
                        Some(crate::parser::lexer::Token::Newline)
                        | Some(crate::parser::lexer::Token::DocumentStart)
                        | Some(crate::parser::lexer::Token::DocumentEnd)
                        | None => Node::None,
                        _ => crate::parser::tokens::value::parse_value_with_tokens(
                            &mut stream,
                            directives,
                            0,
                        )?,
                    }
                    // `stream` (and its mutable borrow of `source`) is dropped here.
                };

                pairs.push((Node::None, value_node));

                // QLJ7: After parsing the value, the lexer may have consumed the
                // trailing newline via lookahead, leaving source positioned at the
                // start of the *next* line — which may be a document-start/end
                // marker (--- or ...). Calling `skip_until_newline` in that state
                // would swallow the marker, preventing the outer document parser
                // from ever seeing a second `---` and allowing undefined tag
                // handles (like `!prefix!`) to silently slip through in subsequent
                // documents. Guard against this by checking whether the current
                // position is already at a `---` or `...` marker before skipping.
                let at_doc_marker = {
                    let st = source.save_state();
                    let c0 = source.current();
                    let c1 = if c0.is_some() {
                        source.next();
                        source.current()
                    } else {
                        None
                    };
                    let c2 = if c1.is_some() {
                        source.next();
                        source.current()
                    } else {
                        None
                    };
                    let c3 = if c2.is_some() {
                        source.next();
                        source.current()
                    } else {
                        None
                    };
                    source.restore_state(st);
                    let is_triple_dash = c0 == Some('-') && c1 == Some('-') && c2 == Some('-');
                    let is_triple_dot = c0 == Some('.') && c1 == Some('.') && c2 == Some('.');
                    let sep_ok = c3.map_or(true, |c| {
                        crate::utils::is_horizontal_space(c)
                            || crate::utils::is_line_terminator(c)
                            || c == '#'
                    });
                    (is_triple_dash || is_triple_dot) && sep_ok
                };
                if !at_doc_marker {
                    crate::utils::skip_until_newline(source);
                    if source
                        .current()
                        .map_or(false, crate::utils::is_line_terminator)
                    {
                        source.next();
                    }
                    crate::utils::skip_whitespace_and_comments(source);
                }
            }
            Ok(crate::parser::utils::node_utils::make_mapping_node(pairs))
        }
        Some(c) if c == '?' => unreachable!(),
        Some(c) if c.is_alphanumeric() => {
            if matches!(head_kind, BlockHeadKind::BlockMapping) {
                let base_indent = source.get_current_indent_level();
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                Ok(parse_mapping_with_tokens(
                    &mut stream,
                    base_indent,
                    directives,
                    0,
                )?)
            } else {
                parse_scalar_or_value(source, directives, indent_level, ctx)
            }
        }
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(
                source,
                indent_level,
                directives,
                ctx,
            )?)
        }
        Some('\0') => {
            source.next();
            Ok(parse_document_contents(
                source,
                indent_level,
                directives,
                ctx,
            )?)
        }
        Some(c) if matches!(c, '<' | '>' | '"' | '\'' | '|') => {
            if matches!(head_kind, BlockHeadKind::BlockMapping) {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                parse_scalar_or_value(source, directives, indent_level, ctx)
            }
        }
        Some('%') => Ok(Node::None),
        Some(c) => {
            let stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            Err(parse_err!(
                &stream,
                &format!(
                    "{}{}",
                    crate::error::messages::ERR_UNEXPECTED_CHAR_PREFIX,
                    c
                )
            ))
        }
        None => Ok(Node::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::Node;
    use crate::parser::directives::DirectiveContext;
    use crate::parser::utils::context::CollectionType;
    use crate::parser::utils::context::ParsingContext;

    fn make_ctx_none() -> ParsingContext {
        ParsingContext {
            collection_type: CollectionType::None,
            ..Default::default()
        }
    }

    #[test]
    fn test_parse_scalar_or_value_plain_multiline() {
        let mut buf = Buffer::new(b"abc\ndef\nghi");
        let directives = DirectiveContext::default();
        let ctx = make_ctx_none();
        let node = parse_scalar_or_value(&mut buf, &directives, 0, &ctx).unwrap();
        assert_eq!(node, Node::from("abc def ghi"));
    }

    #[test]
    fn test_parse_scalar_or_value_value_branch() {
        let mut buf = Buffer::new(b"42");
        let directives = DirectiveContext::default();
        let ctx = ParsingContext {
            collection_type: CollectionType::BlockSequence,
            ..Default::default()
        };
        let node = parse_scalar_or_value(&mut buf, &directives, 0, &ctx).unwrap();
        // Accepts any parse_value result, just check not None
        assert!(node != Node::None);
    }

    #[test]
    fn test_parse_document_contents_unexpected_char() {
        let mut buf = Buffer::new(b"$");
        let directives = DirectiveContext::default();
        let ctx = make_ctx_none();
        let result = parse_document_contents(&mut buf, 0, &directives, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_document_contents_none_on_eof() {
        let mut buf = Buffer::new(b"");
        let directives = DirectiveContext::default();
        let ctx = make_ctx_none();
        let node = parse_document_contents(&mut buf, 0, &directives, &ctx).unwrap();
        assert_eq!(node, Node::None);
    }

    #[test]
    fn test_2cms_regression_az63_should_succeed() {
        // AZ63: simple mapping with sequence values - should succeed
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config("one:\n- 2\n- 3\nfour: 5\n", config);
        assert!(
            result.is_ok(),
            "AZ63 should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_2cms_fix_invalid_key_value_in_continuation() {
        // 2CMS: plain multiline scalar where continuation line looks like mapping entry
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config("this\n is\n  invalid: x\n", config);
        assert!(
            result.is_err(),
            "2CMS should fail: continuation line 'invalid: x' is a mapping entry"
        );
    }

    #[test]
    fn test_c2sp_multiline_flow_seq_implicit_key_errors() {
        // C2SP: [23\n]: 42 — flow sequence spanning two lines used as implicit mapping key
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config("[23\n]: 42\n", config);
        assert!(
            result.is_err(),
            "C2SP: multiline flow sequence as implicit mapping key should error"
        );
    }}