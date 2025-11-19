//! Module: parser/document/scalar.rs

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
pub(crate) fn parse_scalar(value: &str, directives: &DirectiveContext) -> Node {
    match value {
        v if v.starts_with('#') => Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None),
        "null" | "~" => Node::None,
        "true" => Node::Boolean(true),
        "false" => Node::Boolean(false),
        // YAML 1.1 specific boolean values
        "yes" | "Yes" | "YES" if directives.is_yaml_11() => Node::Boolean(true),
        "no" | "No" | "NO" if directives.is_yaml_11() => Node::Boolean(false),
        "on" | "On" | "ON" if directives.is_yaml_11() => Node::Boolean(true),
        "off" | "Off" | "OFF" if directives.is_yaml_11() => Node::Boolean(false),
        v => {
            // Try parsing as octal first (version-specific)
            if v.starts_with('0') && v.len() > 1 {
                // YAML 1.2: 0o prefix for octal
                if v.starts_with("0o") || v.starts_with("0O") {
                    if let Ok(i) = i64::from_str_radix(&v[2..], 8) {
                        return Node::Number(Numeric::Integer(i));
                    }
                }
                // YAML 1.1: plain 0 prefix for octal (e.g., 0755)
                else if directives.is_yaml_11() && v.chars().skip(1).all(|c| c >= '0' && c <= '7')
                {
                    if let Ok(i) = i64::from_str_radix(&v[1..], 8) {
                        return Node::Number(Numeric::Integer(i));
                    }
                }
            }

            // Try standard integer parsing
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
                Node::Str(content, qt, style)
            }
        }
    }
}
