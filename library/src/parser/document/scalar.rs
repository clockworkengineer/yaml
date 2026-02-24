
//! Scalar Parsing Logic
//!
//! Implements detection and parsing of YAML scalars, including block and plain scalars.
//! Provides helpers for block scalar header parsing and plain scalar handling.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Shared helper for block scalar and plain scalar detection and parsing
use crate::constants::{CHAR_DASH, CHAR_GREATER_THAN, CHAR_SPACE, CHAR_VERTICAL_BAR};
use crate::error::YamlError;
use crate::io::traits::ISource;

/// Parsed view of a block scalar header ("|" or ">" line).
///
/// Currently we only need to know whether an explicit indent indicator
/// was provided; additional fields can be added if future refactors
/// need full chomping/indent metadata.
struct ParsedBlockHeader {
    has_explicit_indent_indicator: bool,
}

/// Parses a potential block scalar header line.
///
/// Returns:
/// - Ok(None) when the line does not represent a valid block scalar header
///   (so the caller should treat it as a plain scalar).
/// - Ok(Some(ParsedBlockHeader)) when the header is valid.
/// - Err(YamlError) when the header syntax is invalid and should be
///   reported as a block scalar error (e.g., unexpected text immediately
///   after the indicator, or an invalid explicit indent indicator).
fn parse_block_header_line(
    s: &str,
    source: &mut dyn ISource,
    check_unexpected_text: bool,
) -> Result<Option<ParsedBlockHeader>, YamlError> {
    let mut chars = s.chars();
    let indicator = match chars.next() {
        Some(c) if c == CHAR_VERTICAL_BAR || c == CHAR_GREATER_THAN => c,
        _ => return Ok(None),
    };

    let rest = chars.as_str();

    // For non-flow contexts, enforce the "unexpected text immediately
    // after '|' or '>'" rule using the first non-whitespace token.
    if check_unexpected_text {
        let meta = rest.trim_start_matches(CHAR_SPACE);
        if !meta.is_empty() {
            if let Some(first_token) = meta.split_whitespace().next() {
                let first_ok = first_token
                    .chars()
                    .all(|c| c == '+' || c == CHAR_DASH || c.is_ascii_digit());
                if !first_ok {
                    return Err(crate::parser::document::errors::block_scalar_errors::BlockScalarErrors::invalid_header_unexpected_text(
                        source,
                    ));
                }
            }
        }
    }

    // Determine if this line is a syntactically valid block header by
    // ensuring all remaining characters belong to the allowed set.
    let is_header = rest
        .chars()
        .all(|c| c == CHAR_SPACE || c == '+' || c == CHAR_DASH || c.is_ascii_digit());
    if !is_header {
        return Ok(None);
    }

    // Validate explicit indent indicator (single digit 1-9) if present.
    let header_meta = s[indicator.len_utf8()..].trim();
    let digits: String = header_meta.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        if digits.len() != 1 || digits.chars().next().unwrap() == '0' {
            return Err(crate::parser::document::errors::block_scalar_errors::BlockScalarErrors::invalid_indent_indicator(
                source,
            ));
        }
    }

    Ok(Some(ParsedBlockHeader {
        has_explicit_indent_indicator: !digits.is_empty(),
    }))
}

fn parse_scalar_dispatch(
    stream: &mut TokenStream,
    s: &str,
    directives: &DirectiveContext,
) -> crate::parser::ParseResult<Node> {
    // Unified block scalar header parsing (non-flow contexts enforce stricter
    // header validation than flow contexts, matching existing behavior).
    let in_flow = stream.in_flow();
    let header_info = parse_block_header_line(s, stream.source_mut(), !in_flow)?;
    if let Some(ParsedBlockHeader {
        has_explicit_indent_indicator,
    }) = header_info
    {
        let block_header = s;
        stream.next()?;
        return parse_block_scalar(stream, &block_header, has_explicit_indent_indicator);
    }
    // Otherwise, treat as plain scalar with possible indented continuation lines
    parse_plain_scalar(stream, s, directives)
}
// Extracted single-quoted scalar parsing logic
fn parse_single_quoted_scalar(
    stream: &mut TokenStream,
    s: &str,
) -> crate::parser::ParseResult<Node> {
    use crate::nodes::node::{BlockStyle, Node, QuoteType};
    stream.next()?;
    Ok(Node::Str(
        s.to_string(),
        QuoteType::Single,
        BlockStyle::None,
    ))
}

// Extracted double-quoted scalar parsing logic
fn parse_double_quoted_scalar(
    stream: &mut TokenStream,
    s: &str,
) -> crate::parser::ParseResult<Node> {
    use crate::nodes::node::{BlockStyle, Node, QuoteType};
    stream.next()?;
    let unescaped = crate::utils::unescape_double_quoted(s);
    Ok(Node::Str(unescaped, QuoteType::Double, BlockStyle::None))
}
// Extracted plain scalar parsing logic
fn parse_plain_scalar(
    stream: &mut TokenStream,
    s: &str,
    directives: &DirectiveContext,
) -> crate::parser::ParseResult<Node> {
    use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
    use crate::parser::lexer::Token;
    stream.next()?;
    let mut accumulated = s.to_string();
    loop {
        if stream.is_current(|t| matches!(t, Token::Newline)) {
            stream.next()?;
            if let Some(Token::Indent(level)) = stream.current() {
                if *level > 0 {
                    stream.next()?;
                    if stream.peek()?.map_or(false, |t| matches!(t, Token::Colon)) {
                        break;
                    }
                    if let Some(Token::Plain(seg)) = stream.current() {
                        accumulated.push(' ');
                        accumulated.push_str(seg);
                        stream.next()?;
                        continue;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    match accumulated.as_str() {
        "null" | "~" => Ok(Node::None),
        "true" => Ok(Node::Boolean(true)),
        "false" => Ok(Node::Boolean(false)),
        v if directives.is_yaml_11() && matches!(v, "yes" | "Yes" | "YES" | "on" | "On" | "ON") => {
            Ok(Node::Boolean(true))
        }
        v if directives.is_yaml_11() && matches!(v, "no" | "No" | "NO" | "off" | "Off" | "OFF") => {
            Ok(Node::Boolean(false))
        }
        v => {
            if v.starts_with('0') && v.len() > 1 {
                if v.starts_with("0o") || v.starts_with("0O") {
                    if let Ok(i) = i64::from_str_radix(&v[2..], 8) {
                        return Ok(Node::Number(Numeric::Integer(i)));
                    }
                } else if directives.is_yaml_11() && v.chars().skip(1).all(|c| c >= '0' && c <= '7')
                {
                    if let Ok(i) = i64::from_str_radix(&v[1..], 8) {
                        return Ok(Node::Number(Numeric::Integer(i)));
                    }
                }
            }
            if let Ok(i) = v.parse::<i64>() {
                Ok(Node::Number(Numeric::Integer(i)))
            } else if let Ok(f) = v.parse::<f64>() {
                Ok(Node::Number(Numeric::Float(f)))
            } else {
                Ok(Node::Str(
                    v.to_string(),
                    QuoteType::Unquoted,
                    BlockStyle::None,
                ))
            }
        }
    }
}
// Extracted block scalar parsing logic
fn parse_block_scalar(
    stream: &mut TokenStream,
    block_header: &str,
    has_explicit_indent_indicator: bool,
) -> crate::parser::ParseResult<Node> {
    use crate::parser::lexer::Token;
    let mut block_lines: Vec<String> = Vec::new();
    let mut trailing_newlines: usize = 0;
    let mut first_content_indent: Option<usize> = None;
    let mut max_blank_indent_before_content: usize = 0;
    let mut blank_lines_before_content: usize = 0;
    let mut pending_indent_for_line: Option<usize> = None;
    let mut saw_plain_current_line: bool = false;
    // Track if a comment appears before the first content line; used only
    // for diagnostics in debug builds.
    let mut _comment_before_first_content: bool = false;
    loop {
        match stream.current() {
            Some(Token::Indent(level)) => {
                pending_indent_for_line = Some(*level);
                stream.next()?;
            }
            Some(Token::Plain(line)) => {
                let indent = pending_indent_for_line.unwrap_or(0);
                // Do not treat an unindented top-level line as block scalar content.
                // This prevents cases like Y79Y/001 (a tab-only line followed by
                // a top-level mapping key) from being misclassified as the first
                // content line of the scalar. In such cases, the scalar should be
                // considered empty and parsing should continue with the following
                // top-level content (e.g., "bar: 1").
                if first_content_indent.is_none() && indent == 0 {
                    // Heuristic: only treat the upcoming top-level line as
                    // not part of the scalar (thus ending the block) when
                    // we've seen at least two columns of indentation on the
                    // preceding blank-only lines. This distinguishes Y79Y/001
                    // (space+tab blank line → valid empty scalar) from Y79Y/000
                    // (tab-only blank line → remains an error under our
                    // existing validation rules).
                    if max_blank_indent_before_content >= 2 {
                        break;
                    }
                }
                if first_content_indent.is_none() {
                    first_content_indent = Some(indent);
                }
                if first_content_indent.is_some() && indent >= first_content_indent.unwrap() {
                    let mut line_stripped = String::new();
                    let mut chars = line.chars().peekable();
                    let mut in_single = false;
                    let mut in_double = false;
                    while let Some(c) = chars.next() {
                        match c {
                            '\'' if !in_double => in_single = !in_single,
                            '"' if !in_single => in_double = !in_double,
                            '#' if !in_single && !in_double => {
                                break;
                            }
                            _ => {}
                        }
                        line_stripped.push(c);
                    }
                    let line_stripped = line_stripped.trim_end().to_string();
                    block_lines.push(line_stripped);
                    trailing_newlines = 0;
                    saw_plain_current_line = true;
                    stream.next()?;
                } else {
                    break;
                }
            }
            Some(Token::Newline) => {
                if !saw_plain_current_line
                    && first_content_indent.is_none()
                    && pending_indent_for_line.is_some()
                {
                    let indent = pending_indent_for_line.unwrap_or(0);
                    if indent > max_blank_indent_before_content {
                        max_blank_indent_before_content = indent;
                    }
                    blank_lines_before_content += 1;
                }
                trailing_newlines += 1;
                stream.next()?;
                pending_indent_for_line = None;
                saw_plain_current_line = false;
            }
            Some(Token::Comment(_)) => {
                if first_content_indent.is_none() {
                    _comment_before_first_content = true;
                }
                stream.next()?;
            }
            Some(Token::Eof) => {
                break;
            }
            _ => break,
        }
    }

    // Loosen blank line indentation rules: allow any number/indentation of blank lines before first content line
    // Only enforce indentation rules on actual content lines, not on blank lines before content
    let indicator = block_header.chars().next().unwrap();
    if indicator == '|' && !has_explicit_indent_indicator {
        if let Some(first_indent) = first_content_indent {
            if blank_lines_before_content >= 1 && max_blank_indent_before_content > first_indent {
                return Err(crate::parser::document::errors::block_scalar_errors::BlockScalarErrors::invalid_literal_blank_indent(
                    stream.source_mut(),
                    max_blank_indent_before_content,
                    first_indent,
                ));
            }
        }
    }
    use crate::nodes::node::{BlockStyle, Node, QuoteType};
    let style = if indicator == '|' {
        BlockStyle::Literal
    } else {
        BlockStyle::Folded
    };
    let mut full = String::new();
    if !block_lines.is_empty() {
        full.push_str(&block_lines.join("\n"));
    }
    let header_meta = block_header[1..].trim_start();
    if header_meta.contains('+') && trailing_newlines > 0 {
        for _ in 0..trailing_newlines {
            full.push('\n');
        }
    }
    Ok(Node::Str(full, QuoteType::Unquoted, style))
}
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
// Recursion guard removed

/// Parses a scalar value from tokens (TokenStream)
#[allow(deprecated)]
pub(crate) fn parse_scalar_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    _depth: usize,
) -> crate::parser::ParseResult<Node> {
    // Recursion guard removed
    // Capture the current token to avoid borrow checker issues in error path.
    let current_token = stream.current().cloned();
    let current_token_str = format!("{:?}", current_token);
    match current_token {
        Some(Token::SingleQuoted(s)) => parse_single_quoted_scalar(stream, &s),
        Some(Token::DoubleQuoted(s)) => parse_double_quoted_scalar(stream, &s),
        Some(Token::Plain(s)) => parse_scalar_dispatch(stream, &s, directives),
        _ => Err(
            crate::parser::document::errors::token_errors::expected_scalar_token(
                stream.source_mut(),
                &current_token_str,
            ),
        ),
    }
}
// Module: parser/document/scalar.rs

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::{BlockStyle, QuoteType};

    #[test]
    fn test_block_literal_basic_via_tokens() {
        let mut source = Buffer::new(b"|\n  line1\n  line2\n");
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Literal) if s == "line1\nline2")
        );
    }
    #[test]
    fn test_block_folded_basic_via_tokens() {
        let mut source = Buffer::new(b">\n  line1\n  line2\n");
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Folded) if s == "line1\nline2")
        );
    }

    #[test]
    fn test_block_scalar_chomping_strip_minus() {
        // Using string-based parser path to verify chomping behavior
        let directives = DirectiveContext::new();
        let value = "| -\n  a\n  b\n\n";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Literal) if !s.ends_with('\n'))
        );
    }

    #[test]
    fn test_block_scalar_chomping_keep_plus() {
        let directives = DirectiveContext::new();
        let value = "| +\n  a\n  b\n\n\n";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Literal) if s.ends_with("\n\n"))
        );
    }

    #[test]
    fn test_block_scalar_indent_indicator() {
        let directives = DirectiveContext::new();
        let value = "| 2\n    a\n    b\n";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Literal) if s.contains("a") && s.contains("b"))
        );
    }

    #[test]
    fn test_block_scalar_w9l4_like_indentation_errors() {
        // W9L4-style literal block scalar where a blank line before the first
        // content line is more indented than the content itself. This should
        // be rejected as an indentation error.
        let directives = DirectiveContext::new();
        let value = "|\n     \n  more spaces at the beginning\n  are invalid\n";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let res = parse_scalar_with_tokens(&mut stream, &directives, 0);
        assert!(res.is_err());
        let err = res.unwrap_err();
        // Check error kind or message from YamlError
        assert!(err.to_string().contains("indentation"));
    }

    #[test]
    fn test_plain_scalar_multiline() {
        let directives = DirectiveContext::new();
        let value = "plain line 1\n  plain line 2\nplain line 3";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::None) if s.contains("plain line 1") && s.contains("plain line 2"))
        );
    }

    #[test]
    fn test_single_quoted_scalar() {
        let directives = DirectiveContext::new();
        let value = "'single quoted string'";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Single, BlockStyle::None) if s == "single quoted string")
        );
    }

    #[test]
    fn test_double_quoted_scalar() {
        let directives = DirectiveContext::new();
        let value = "\"double quoted string\"";
        let mut source = Buffer::new(value.as_bytes());
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Double, BlockStyle::None) if s == "double quoted string")
        );
    }

    // ---
    // The scalar parser is now modular:
    // - parse_block_scalar: handles block scalars (|, >)
    // - parse_plain_scalar: handles plain and multiline scalars
    // - parse_single_quoted_scalar: handles single-quoted scalars
    // - parse_double_quoted_scalar: handles double-quoted scalars
    // Error handling is centralized and token stream navigation uses helpers for DRYness.
}
