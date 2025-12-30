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
            // Block scalar detection: treat as block scalar ONLY when the plain token is a valid block header line
            // i.e., first char '|' or '>' and the remainder contains only spaces, '+', '-', or digits (indent hints).
            // This avoids misinterpreting plain tokens like ">folded" inside flow collections.
            let is_block_header = {
                let mut chars = s.chars();
                if let Some(ind) = chars.next() {
                    if ind == '|' || ind == '>' {
                        let rest = chars.as_str();
                        rest.chars()
                            .all(|c| c == ' ' || c == '+' || c == '-' || c.is_ascii_digit())
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if is_block_header {
                let _indicator = s.chars().next().unwrap();
                // Validate indentation indicator range if present: must be a single digit 1-9
                let header_meta = s[1..].trim();
                let digits: String = header_meta
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if !digits.is_empty() {
                    if digits.len() != 1 || digits.chars().next().unwrap() == '0' {
                        return Err("Invalid block scalar indentation indicator: must be a single digit from 1-9".to_string());
                    }
                }
                let block_header = s.clone();
                stream.next()?;
                let mut block_lines: Vec<String> = Vec::new();
                let mut trailing_newlines: usize = 0;
                // Collect subsequent lines: consume indentation tokens, capture plain content lines,
                // and ignore standalone newline tokens (they serve as separators).
                loop {
                    match stream.current() {
                        Some(Token::Indent(_)) => {
                            stream.next()?;
                        }
                        Some(Token::Plain(line)) => {
                            block_lines.push(line.clone());
                            trailing_newlines = 0; // reset when we see content
                            stream.next()?;
                        }
                        Some(Token::Newline) => {
                            // Treat as separator; count for trailing chomping semantics
                            trailing_newlines += 1;
                            stream.next()?;
                        }
                        _ => break,
                    }
                }
                // Build result as header + newline + joined lines; no folding here.
                let indicator = block_header.chars().next().unwrap();
                let style = if indicator == '|' {
                    BlockStyle::Literal
                } else {
                    BlockStyle::Folded
                };
                let mut full = block_header.trim_end().to_string();
                if !block_lines.is_empty() {
                    full.push('\n');
                    full.push_str(&block_lines.join("\n"));
                }
                // Apply simplified chomping: keep '+' preserves trailing newlines, '-' strips (default already stripped)
                let header_meta = block_header[1..].trim_start();
                if header_meta.contains('+') && trailing_newlines > 0 {
                    for _ in 0..trailing_newlines {
                        full.push('\n');
                    }
                }
                return Ok(Node::Str(full, QuoteType::Unquoted, style));
            }
            // Otherwise, treat as plain scalar with possible indented continuation lines
            stream.next()?;
            let mut accumulated = s.clone();
            // Handle multiline plain scalars: if a newline is followed by an indented plain line,
            // treat it as a continuation separated by a space. We advance tokens in a
            // forward-only manner; consuming a newline/indent without a following plain
            // is acceptable since callers generally skip trivia.
            loop {
                match stream.current() {
                    Some(Token::Newline) => {
                        // Consume the newline to inspect the next line
                        stream.next()?;
                        // If the next token is indentation > 0, consume it
                        match stream.current() {
                            Some(Token::Indent(level)) if *level > 0 => {
                                stream.next()?; // consume indent
                                // If this plain token is followed by a colon, it's a mapping key,
                                // not a continuation of the previous scalar. Do not consume it.
                                if matches!(stream.peek()?, Some(Token::Colon)) {
                                    break;
                                }
                                if let Some(Token::Plain(seg)) = stream.current() {
                                    // Continuation line
                                    accumulated.push(' ');
                                    accumulated.push_str(seg);
                                    stream.next()?; // consume plain segment
                                    continue; // attempt to gather further continuation lines
                                } else {
                                    // Not a plain segment; stop accumulating
                                    break;
                                }
                            }
                            _ => {
                                // No indentation after newline -> not a continuation
                                break;
                            }
                        }
                    }
                    _ => break,
                }
            }
            match accumulated.as_str() {
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
        _ => Err(format!(
            "Expected a scalar token, got {:?}", stream.current()
        )),
    }
}
// Module: parser/document/scalar.rs

use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, Numeric, QuoteType};
use crate::parser::directives::DirectiveContext;

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
