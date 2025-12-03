//! Helper functions for parsing block scalars (literal | and folded >)

use crate::nodes::node::{BlockStyle, Node, QuoteType};
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Parses a block scalar (literal | or folded >) using tokens
/// Returns a tuple of (content_string, is_folded)
pub(crate) fn parse_block_scalar(
    stream: &mut TokenStream,
    is_folded: bool,
) -> Result<String, String> {
    let mut lines: Vec<String> = Vec::new();
    let mut first_indent: Option<usize> = None;

    // Skip the initial indicator token (| or >) and any modifiers
    stream.next()?;
    stream.skip_whitespace()?;

    while let Some(token) = stream.current() {
        match token {
            Token::Newline => {
                stream.next()?;
                lines.push(String::new());
                continue;
            }
            Token::Indent(level) => {
                if first_indent.is_none() {
                    first_indent = Some(*level);
                } else if *level < first_indent.unwrap() {
                    break;
                }
                stream.next()?;
                continue;
            }
            Token::Plain(s) => {
                lines.push(s.clone());
                stream.next()?;
                continue;
            }
            Token::Eof => break,
            _ => {
                stream.next()?;
            }
        }
    }

    // Remove trailing empty lines
    while matches!(lines.last(), Some(s) if s.is_empty()) {
        lines.pop();
    }

    let fi = first_indent.unwrap_or(0);
    let mut norm_lines: Vec<String> = Vec::with_capacity(lines.len());

    if is_folded {
        // Folded scalar: normalize indentation
        for l in lines.iter() {
            if l.is_empty() {
                norm_lines.push(String::new());
            } else {
                let lead = l.chars().take_while(|&ch| ch == ' ').count();
                let strip = fi.min(lead);
                let stripped: String = l.chars().skip(strip).collect();
                norm_lines.push(stripped);
            }
        }
    } else {
        // Literal scalar: keep as-is
        norm_lines = lines.clone();
    }

    Ok(norm_lines.join("\n"))
}

/// Creates a block scalar node from parsed content
#[allow(dead_code)]
pub(crate) fn make_block_scalar_node(content: String, is_folded: bool) -> Node {
    let style = if is_folded {
        BlockStyle::Folded
    } else {
        BlockStyle::Literal
    };
    Node::Str(content, QuoteType::Unquoted, style)
}
