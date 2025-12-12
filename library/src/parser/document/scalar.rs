use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
// Recursion guard removed

/// Parses a scalar value from tokens (TokenStream)
#[allow(deprecated)]
pub(crate) fn parse_scalar_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    _depth: usize,
) -> Result<Node, String> {
    // Recursion guard removed
    // Clone the current token to avoid holding an immutable borrow while advancing the stream.
    let current = stream.current().cloned();
    match current {
        Some(Token::SingleQuoted(s)) => {
            stream.next()?;
            Ok(Node::Str(s, QuoteType::Single, BlockStyle::None))
        }
        Some(Token::DoubleQuoted(s)) => {
            stream.next()?;
            let unescaped = crate::utils::unescape_double_quoted(&s);
            Ok(Node::Str(unescaped, QuoteType::Double, BlockStyle::None))
        }
        Some(Token::Plain(s)) => {
            // Block scalar detection: if plain token starts with | or >, collect all indented/plain lines as content
            if s.starts_with('|') || s.starts_with('>') {
                let indicator = s.chars().next().unwrap();
                let mut block_header = s.clone();
                stream.next()?;
                let mut block_lines: Vec<String> = Vec::new();
                // Collect all subsequent indented/plain lines as block scalar content
                loop {
                    // Accept Indent, Plain, and Newline tokens as part of the block scalar
                    match stream.current() {
                        Some(Token::Indent(_)) => {
                            stream.next()?;
                        }
                        Some(Token::Plain(line)) => {
                            block_lines.push(line.clone());
                            stream.next()?;
                        }
                        Some(Token::Newline) => {
                            block_lines.push(String::new());
                            stream.next()?;
                        }
                        _ => break,
                    }
                }
                // Reconstruct the block scalar string as if it was a single string after the indicator
                let mut v = block_header;
                if !block_lines.is_empty() {
                    v.push('\n');
                    v.push_str(&block_lines.join("\n"));
                }
                // Now use the old block scalar logic to process v
                let mut chars = v.chars();
                let indicator = chars.next().unwrap();
                let mut chomping = None;
                let mut indent_str = String::new();
                let mut rest = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '+' | '-' if chomping.is_none() => chomping = Some(c),
                        d if d.is_ascii_digit() => {
                            indent_str.push(d);
                        }
                        ' ' => continue,
                        _ => {
                            rest.push(c);
                            rest.push_str(chars.as_str());
                            break;
                        }
                    }
                }
                if !indent_str.is_empty() {
                    if indent_str == "0" {
                        return Err("Invalid block scalar indentation indicator: 0. Only 1-9 allowed. See YAML spec. Error: indentation indicator must be 1-9".to_string());
                    }
                    if indent_str.len() > 1 {
                        return Err(format!(
                            "Invalid block scalar indentation indicator: {}. Only single digit 1-9 allowed. Error: indentation indicator must be 1-9, single digit",
                            indent_str
                        ));
                    }
                }
                let indent = if indent_str.is_empty() {
                    None
                } else {
                    indent_str.parse::<usize>().ok()
                };
                let content = if rest.is_empty() {
                    v[1..].trim_start()
                } else {
                    rest.trim_start()
                };
                let lines: Vec<&str> = content.lines().collect();
                let min_indent = indent.unwrap_or_else(|| {
                    lines
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| l.chars().take_while(|c| *c == ' ').count())
                        .min()
                        .unwrap_or(0)
                });
                let stripped: Vec<&str> = lines
                    .iter()
                    .map(|l| {
                        if l.len() >= min_indent {
                            &l[min_indent..]
                        } else {
                            l.trim_end()
                        }
                    })
                    .collect();
                let style = if indicator == '|' {
                    BlockStyle::Literal
                } else {
                    BlockStyle::Folded
                };
                let mut result = String::new();
                if style == BlockStyle::Literal {
                    result = stripped.join("\n");
                } else {
                    let mut prev_blank = false;
                    for line in &stripped {
                        if line.trim().is_empty() {
                            result.push('\n');
                            prev_blank = true;
                        } else {
                            if !result.is_empty() && !prev_blank {
                                result.push(' ');
                            }
                            result.push_str(line);
                            prev_blank = false;
                        }
                    }
                }
                let trailing_newlines = content.chars().rev().take_while(|c| *c == '\n').count();
                match chomping {
                    Some('+') => {
                        for _ in 0..trailing_newlines {
                            result.push('\n');
                        }
                    }
                    Some('-') => {
                        result = result.trim_end_matches('\n').to_string();
                    }
                    _ => {
                        if trailing_newlines > 0 {
                            result.push('\n');
                        }
                    }
                }
                return Ok(Node::Str(result, QuoteType::Unquoted, style));
            }
            // Otherwise, treat as plain scalar
            stream.next()?;
            match s.as_str() {
                "null" | "~" => Ok(Node::None),
                "true" => Ok(Node::Boolean(true)),
                "false" => Ok(Node::Boolean(false)),
                v if directives.is_yaml_11() && matches!(v, "yes"|"Yes"|"YES"|"on"|"On"|"ON") => Ok(Node::Boolean(true)),
                v if directives.is_yaml_11() && matches!(v, "no"|"No"|"NO"|"off"|"Off"|"OFF") => Ok(Node::Boolean(false)),
                v => {
                    if v.starts_with('0') && v.len() > 1 {
                        if v.starts_with("0o") || v.starts_with("0O") {
                            if let Ok(i) = i64::from_str_radix(&v[2..], 8) {
                                return Ok(Node::Number(Numeric::Integer(i)));
                            }
                        } else if directives.is_yaml_11() && v.chars().skip(1).all(|c| c >= '0' && c <= '7') {
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
                        Ok(Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None))
                    }
                }
            }
        }
        _ => Err("Expected a scalar token".to_string()),
    }
}
// Module: parser/document/scalar.rs

use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, Numeric, QuoteType};
use crate::parser::directives::DirectiveContext;

/// Parses a scalar value string into the appropriate YAML Node type.
///
/// Handles various scalar types including null values, booleans, numbers,
/// and strings. Processes quoted strings (single and double) with proper
/// escape sequence handling and quote style preservation. Determines the
/// appropriate BlockStyle and QuoteType based on content analysis.
///
/// Version-specific behavior:
/// - YAML 1.1: Accepts yes/no/on/off as booleans, octal with 0 prefix
/// - YAML 1.2: Only true/false as booleans, octal with 0o prefix
///
/// # Arguments
///
/// * `value` - The string value to parse as a scalar
/// * `directives` - Directive context for version-aware parsing
///
/// # Returns
///
/// A Node representing the parsed scalar value
#[deprecated(note = "Use parse_scalar_with_tokens instead")]
pub(crate) fn parse_scalar(value: &str, directives: &DirectiveContext) -> Result<Node, String> {
    match value {
        v if v.starts_with('#') => Ok(Node::Str(
            v.to_string(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        "null" | "~" => Ok(Node::None),
        "true" => Ok(Node::Boolean(true)),
        "false" => Ok(Node::Boolean(false)),
        // YAML 1.1 specific boolean values
        "yes" | "Yes" | "YES" if directives.is_yaml_11() => Ok(Node::Boolean(true)),
        "no" | "No" | "NO" if directives.is_yaml_11() => Ok(Node::Boolean(false)),
        "on" | "On" | "ON" if directives.is_yaml_11() => Ok(Node::Boolean(true)),
        "off" | "Off" | "OFF" if directives.is_yaml_11() => Ok(Node::Boolean(false)),
        v => {
            // Only treat | or > as block scalar indicators in block context, not in flow collections
            // For flow context, these should be part of the string
            // (Assume flow context if there are no newlines in the value)
            if (v.starts_with('|') || v.starts_with('>')) && v.contains('\n') {
                // Block scalar indicator: | or >, possibly followed by chomping (+/-) and indentation
                let mut chars = v.chars();
                let indicator = chars.next().unwrap();
                let mut chomping = None;
                let mut indent_str = String::new();
                let mut rest = String::new();
                // Parse optional chomping and indentation
                while let Some(c) = chars.next() {
                    match c {
                        '+' | '-' if chomping.is_none() => chomping = Some(c),
                        d if d.is_ascii_digit() => {
                            indent_str.push(d);
                        }
                        ' ' => continue,
                        _ => {
                            rest.push(c);
                            rest.push_str(chars.as_str());
                            break;
                        }
                    }
                }
                // Validate indentation indicator
                if !indent_str.is_empty() {
                    if indent_str == "0" {
                        return Err("Invalid block scalar indentation indicator: 0. Only 1-9 allowed. See YAML spec. Error: indentation indicator must be 1-9".to_string());
                    }
                    if indent_str.len() > 1 {
                        return Err(format!(
                            "Invalid block scalar indentation indicator: {}. Only single digit 1-9 allowed. Error: indentation indicator must be 1-9, single digit",
                            indent_str
                        ));
                    }
                }
                let indent = if indent_str.is_empty() {
                    None
                } else {
                    indent_str.parse::<usize>().ok()
                };
                let content = if rest.is_empty() {
                    v[1..].trim_start()
                } else {
                    rest.trim_start()
                };

                // Split lines and apply indentation
                let lines: Vec<&str> = content.lines().collect();
                let min_indent = indent.unwrap_or_else(|| {
                    lines
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| l.chars().take_while(|c| *c == ' ').count())
                        .min()
                        .unwrap_or(0)
                });
                let stripped: Vec<&str> = lines
                    .iter()
                    .map(|l| {
                        if l.len() >= min_indent {
                            &l[min_indent..]
                        } else {
                            l.trim_end()
                        }
                    })
                    .collect();

                let style = if indicator == '|' {
                    BlockStyle::Literal
                } else {
                    BlockStyle::Folded
                };
                let mut result = String::new();
                if style == BlockStyle::Literal {
                    result = stripped.join("\n");
                } else {
                    // Folded: replace newlines with space except for empty lines
                    let mut prev_blank = false;
                    for line in &stripped {
                        if line.trim().is_empty() {
                            result.push('\n');
                            prev_blank = true;
                        } else {
                            if !result.is_empty() && !prev_blank {
                                result.push(' ');
                            }
                            result.push_str(line);
                            prev_blank = false;
                        }
                    }
                }

                // Chomping: '+' (keep all), '-' (strip all), default (keep one)
                let trailing_newlines = content.chars().rev().take_while(|c| *c == '\n').count();
                match chomping {
                    Some('+') => {
                        for _ in 0..trailing_newlines {
                            result.push('\n');
                        }
                    }
                    Some('-') => {
                        // Strip all trailing newlines
                        result = result.trim_end_matches('\n').to_string();
                    }
                    _ => {
                        // Default: keep one trailing newline
                        if trailing_newlines > 0 {
                            result.push('\n');
                        }
                    }
                }
                return Ok(Node::Str(result, QuoteType::Unquoted, style));
            }
            // Try parsing as octal first (version-specific)
            if v.starts_with('0') && v.len() > 1 {
                // YAML 1.2: 0o prefix for octal
                if v.starts_with("0o") || v.starts_with("0O") {
                    if let Ok(i) = i64::from_str_radix(&v[2..], 8) {
                        return Ok(Node::Number(Numeric::Integer(i)));
                    }
                }
                // YAML 1.1: plain 0 prefix for octal (e.g., 0755)
                else if directives.is_yaml_11() && v.chars().skip(1).all(|c| c >= '0' && c <= '7')
                {
                    if let Ok(i) = i64::from_str_radix(&v[1..], 8) {
                        return Ok(Node::Number(Numeric::Integer(i)));
                    }
                }
            }

            // Try standard integer parsing
            if let Ok(i) = v.parse::<i64>() {
                Ok(Node::Number(Numeric::Integer(i)))
            } else if let Ok(f) = v.parse::<f64>() {
                Ok(Node::Number(Numeric::Float(f)))
            } else {
                let (content, qt, style) = if v.len() >= 2 {
                    let first = v.chars().next().unwrap();
                    let last = v.chars().next_back().unwrap();
                    if first == '\'' && last == '\'' {
                        let stripped = v[1..v.len() - 1].replace("''", "'");
                        (stripped, QuoteType::Single, BlockStyle::None)
                    } else if first == '"' && last == '"' {
                        let inner = &v[1..v.len() - 1];
                        // Check if the string contains actual newlines in the source (multiline double-quoted string)
                        // vs. newlines from escape sequences like \n
                        let has_source_newlines = inner.contains('\n');
                        let unescaped = crate::utils::unescape_double_quoted(inner);

                        // Only apply folding rules if the original source had actual newlines
                        // Escape sequences like \n should produce literal characters without folding
                        let folded = if has_source_newlines {
                            let mut folded = String::with_capacity(unescaped.len());
                            let mut chars = unescaped.chars().peekable();
                            while let Some(ch) = chars.next() {
                                if ch == '\n' {
                                    // Skip leading whitespace and count empty lines
                                    let mut empty_line_count = 0;
                                    loop {
                                        // Skip whitespace
                                        while let Some(&next_ch) = chars.peek() {
                                            if next_ch == ' ' || next_ch == '\t' {
                                                chars.next();
                                            } else {
                                                break;
                                            }
                                        }

                                        // Check if this is an empty line
                                        if chars.peek() == Some(&'\n') {
                                            empty_line_count += 1;
                                            chars.next(); // Consume the newline
                                        } else {
                                            break;
                                        }
                                    }

                                    // Apply folding rules based on empty lines
                                    if empty_line_count > 0 {
                                        // One or more empty lines: preserve as single line break
                                        folded.push('\n');
                                    } else if chars.peek().is_none() {
                                        // End of string after whitespace
                                        // Don't add anything
                                    } else {
                                        // Non-empty continuation line with no empty lines: fold to space
                                        if !folded.ends_with(' ') && !folded.is_empty() {
                                            folded.push(' ');
                                        }
                                    }
                                } else {
                                    folded.push(ch);
                                }
                            }
                            folded.trim_end().to_string()
                        } else {
                            // No source newlines - just use unescaped string as-is
                            unescaped
                        };

                        let simple = folded.chars().all(|ch| {
                            ch.is_alphanumeric()
                                || ch.is_whitespace()
                                || ch == '.'
                                || (ch as u32) >= 0x80
                        });
                        let has_unicode_escape = inner.contains("\\u");
                        let qt = if !has_source_newlines && has_unicode_escape && simple {
                            QuoteType::Unquoted
                        } else {
                            QuoteType::Double
                        };
                        let style = if inner.ends_with("\\n") {
                            BlockStyle::Literal
                        } else {
                            BlockStyle::None
                        };
                        (folded, qt, style)
                    } else {
                        (v.to_string(), QuoteType::Unquoted, BlockStyle::None)
                    }
                } else {
                    (v.to_string(), QuoteType::Unquoted, BlockStyle::None)
                };
                Ok(Node::Str(content, qt, style))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_block_literal_basic_via_tokens() {
        let mut source = Buffer::new(b"|\n  line1\n  line2\n");
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Literal) if s == "|\nline1\nline2")
        );
    }

    #[test]
    fn test_block_folded_basic_via_tokens() {
        let mut source = Buffer::new(b">\n  line1\n  line2\n");
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();
        let node = parse_scalar_with_tokens(&mut stream, &directives, 0).unwrap();
        assert!(
            matches!(node, Node::Str(ref s, QuoteType::Unquoted, BlockStyle::Folded) if s == ">\nline1\nline2")
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
}
