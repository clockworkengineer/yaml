//! Module: parser/document/scalar.rs

use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, Numeric, QuoteType};

/// Parses a scalar value string into the appropriate YAML Node type.
///
/// Handles various scalar types including null values, booleans, numbers,
/// and strings. Processes quoted strings (single and double) with proper
/// escape sequence handling and quote style preservation. Determines the
/// appropriate BlockStyle and QuoteType based on content analysis.
///
/// # Arguments
///
/// * `value` - The string value to parse as a scalar
///
/// # Returns
///
/// A Node representing the parsed scalar value
pub(crate) fn parse_scalar(value: &str) -> Node {
    match value {
        v if v.starts_with('#') => Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None),
        "null" | "~" => Node::None,
        "true" => Node::Boolean(true),
        "false" => Node::Boolean(false),
        v => {
            if let Ok(i) = v.parse::<i64>() {
                Node::Number(Numeric::Integer(i))
            } else if let Ok(f) = v.parse::<f64>() {
                Node::Number(Numeric::Float(f))
            } else {
                let (content, qt, style) = if v.len() >= 2 {
                    let first = v.chars().next().unwrap();
                    let last = v.chars().next_back().unwrap();
                    if first == '\'' && last == '\'' {
                        let stripped = v[1..v.len() - 1].replace("''", "'");
                        (stripped, QuoteType::Single, BlockStyle::None)
                    } else if first == '"' && last == '"' {
                        let inner = &v[1..v.len() - 1];
                        let unescaped = crate::utils::unescape_double_quoted(inner);
                        let mut folded = String::with_capacity(unescaped.len());
                        let mut chars = unescaped.chars().peekable();
                        let mut saw_multiline = false;
                        while let Some(ch) = chars.next() {
                            if ch == '\n' {
                                saw_multiline = true;
                                let mut space_count = 0usize;
                                while let Some(' ') = chars.peek().copied() {
                                    space_count += 1;
                                    chars.next();
                                }
                                if space_count > 0 {
                                    if !folded.ends_with(' ') && !folded.is_empty() {
                                        folded.push(' ');
                                    }
                                } else if chars.peek().is_some() {
                                    folded.push('\n');
                                }
                            } else {
                                folded.push(ch);
                            }
                        }
                        let mut folded = folded.trim_end().to_string();
                        if saw_multiline && folded.ends_with("\\n") {
                            folded.truncate(folded.len() - 2);
                        }
                        let simple = folded.chars().all(|ch| {
                            ch.is_alphanumeric()
                                || ch.is_whitespace()
                                || ch == '.'
                                || (ch as u32) >= 0x80
                        });
                        let has_unicode_escape = inner.contains("\\u");
                        let qt = if !saw_multiline && has_unicode_escape && simple {
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
                Node::Str(content, qt, style)
            }
        }
    }
}
