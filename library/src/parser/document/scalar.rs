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
                let _indicator = s.chars().next().unwrap();
                let block_header = s.clone();
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
