//! Module: stringify/default.rs

use crate::constants::*;
use crate::io::traits::IDestination;
use crate::nodes::node::*;

/// Escapes special characters in a string for double-quoted YAML representation.
///
/// Processes characters that need escaping in double-quoted strings including
/// newlines, carriage returns, tabs, and backslashes. Preserves existing
/// escape sequences when appropriate.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new String with appropriate escape sequences
fn escape_double(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut iter = s.chars().peekable();
    while let Some(c) = iter.next() {
        match c {
            CHAR_NEWLINE => out.push(CHAR_NEWLINE),
            CHAR_CARRIAGE_RETURN => {
                out.push(CHAR_BACKSLASH);
                out.push('r');
            }
            CHAR_TAB => {
                out.push(CHAR_BACKSLASH);
                out.push('t');
            }
            CHAR_BACKSLASH => {
                if let Some(&next) = iter.peek() {
                    match next {
                        'n' | 'r' | 't' | 'b' | 'x' => {
                            out.push(CHAR_BACKSLASH);
                            out.push(next);
                            iter.next();
                        }
                        _ => {
                            out.push(CHAR_BACKSLASH);
                            out.push(CHAR_BACKSLASH);
                        }
                    }
                } else {
                    out.push(CHAR_BACKSLASH);
                    out.push(CHAR_BACKSLASH);
                }
            }
            '"' => {
                out.push(CHAR_BACKSLASH);
                out.push(CHAR_DOUBLE_QUOTE);
            }
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

/// Escapes single quotes in a string for single-quoted YAML representation.
///
/// Handles the single quote escaping rule where single quotes are escaped
/// by doubling them ('') in single-quoted YAML strings.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new String with single quotes properly escaped
fn escape_single(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == CHAR_SINGLE_QUOTE {
            out.push(CHAR_SINGLE_QUOTE);
            out.push(CHAR_SINGLE_QUOTE);
        } else {
            out.push(c);
        }
    }
    out
}

/// Normalizes newline characters in a string to use Unix-style line endings.
///
/// Converts Windows-style CRLF sequences to LF for consistent output.
///
/// # Arguments
///
/// * `s` - The string to normalize
///
/// # Returns
///
/// A new String with normalized line endings
fn normalize_newlines(s: &str) -> String {
    s.replace(CHAR_CARRIAGE_RETURN, "")
}

/// Recursively stringifies a YAML node with the specified indentation level.
///
/// Handles all node types including scalars, arrays, mappings, documents,
/// anchors, aliases, and comments. Applies proper indentation and formatting
/// rules based on the node type and content.
///
/// # Arguments
///
/// * `node` - The Node to stringify
/// * `destination` - The output destination for the YAML content
/// * `indent` - The current indentation level (number of spaces)
///
/// # Returns
///
/// Result indicating success or an error string
fn stringify_document_with_indent(
    node: &Node,
    destination: &mut dyn IDestination,
    indent: usize,
) -> Result<(), String> {
    let indent_str = "  ".repeat(indent);
    match node {
        Node::None => destination.add_bytes(&format!("{indent_str}null")),
        Node::Boolean(b) => destination.add_bytes(&format!("{indent_str}{b}")),
        Node::Str(s, qt, style) => {
            let s = normalize_newlines(s);
            match qt {
                QuoteType::Double => destination.add_bytes(&format!(
                    "{}{}{}{}",
                    indent_str,
                    CHAR_DOUBLE_QUOTE,
                    escape_double(&s),
                    CHAR_DOUBLE_QUOTE
                )),
                QuoteType::Single => {
                    if !s.contains(CHAR_NEWLINE)
                        && (s.contains(CHAR_SINGLE_QUOTE) || s.contains(CHAR_BACKSLASH))
                    {
                        destination.add_bytes(&format!(
                            "{}{}{}{}",
                            indent_str,
                            CHAR_DOUBLE_QUOTE,
                            escape_double(&s),
                            CHAR_DOUBLE_QUOTE
                        ))
                    } else {
                        destination.add_bytes(&format!(
                            "{}{}{}{}",
                            indent_str,
                            CHAR_SINGLE_QUOTE,
                            escape_single(&s),
                            CHAR_SINGLE_QUOTE
                        ))
                    }
                }
                QuoteType::Unquoted => {
                    if s.contains(CHAR_NEWLINE) || matches!(style, BlockStyle::Literal) {
                        let is_literal = matches!(style, BlockStyle::Literal);

                        let lines: Vec<&str> = s.split(CHAR_NEWLINE).collect();
                        let needs_indent = if is_literal {
                            lines.iter().any(|l| !l.is_empty() && !l.starts_with(' '))
                        } else {
                            true
                        };
                        let content_indent = if needs_indent {
                            "  ".repeat(indent + 1)
                        } else {
                            String::new()
                        };
                        destination
                            .add_bytes(&format!("{indent_str}{STR_LITERAL_BLOCK}{CHAR_NEWLINE}"));

                        if !s.contains(CHAR_NEWLINE) && is_literal {
                            destination.add_bytes(&format!("{content_indent}{s}{CHAR_NEWLINE}"));
                        } else {
                            for line in lines {
                                if line.is_empty() {
                                    destination.add_bytes(&CHAR_NEWLINE.to_string());
                                } else {
                                    destination.add_bytes(&format!(
                                        "{content_indent}{line}{CHAR_NEWLINE}"
                                    ));
                                }
                            }
                        }
                    } else {
                        destination.add_bytes(&format!("{indent_str}{s}"))
                    }
                }
            }
        }
        Node::Comment(c) => {
            let c = normalize_newlines(c);
            destination.add_bytes(&format!("{indent_str}{CHAR_HASH}{CHAR_SPACE}{c}"))
        }
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&format!("{indent_str}{i}")),
            Numeric::Float(f) => destination.add_bytes(&format!("{indent_str}{f}")),
            _ => destination.add_bytes(&format!("{indent_str}{num:?}")),
        },
        Node::Array(items) => {
            for item in items {
                destination.add_bytes(&format!("{indent_str}{CHAR_DASH}{CHAR_SPACE}"));
                match item {
                    Node::Mapping(_) => {
                        let mut buf = crate::io::destinations::buffer::Buffer::new();
                        stringify_document_with_indent(item, &mut buf, indent + 1)?;
                        let mut out = buf.to_string();
                        let child_indent = "  ".repeat(indent + 1);
                        if out.starts_with(&child_indent) {
                            out = out.split_off(child_indent.len());
                        }
                        destination.add_bytes(&out);
                    }
                    Node::Array(_) => {
                        let mut buf = crate::io::destinations::buffer::Buffer::new();
                        stringify_document_with_indent(item, &mut buf, indent + 1)?;
                        let mut out = buf.to_string();
                        let child_indent = "  ".repeat(indent + 1);
                        if out.starts_with(&child_indent) {
                            out = out.split_off(child_indent.len());
                        }
                        destination.add_bytes(&out);
                    }
                    _ => {
                        stringify_document_with_indent(item, destination, 0)?;
                        destination.add_bytes(&CHAR_NEWLINE.to_string());
                    }
                }
            }
        }

        Node::Set(items) => {
            // Render sets with !!set tag in block format
            destination.add_bytes(&format!("{indent_str}!!set"));
            if items.is_empty() {
                destination.add_bytes(" {}\n");
            } else {
                destination.add_bytes(&CHAR_NEWLINE.to_string());
                for item in items {
                    match item {
                        // Complex items (mappings, arrays, sets) require explicit-key style
                        Node::Mapping(_) | Node::Array(_) | Node::Set(_) => {
                            destination.add_bytes(&format!("{indent_str}? "));
                            destination.add_bytes(&CHAR_NEWLINE.to_string());
                            stringify_document_with_indent(item, destination, indent + 1)?;
                        }
                        // Simple scalar keys can be rendered as key: null entries which are
                        // easier for the parser to round-trip.
                        _ => {
                            // Serialize the key into a temporary buffer to obtain its text
                            let mut key_buf = crate::io::destinations::buffer::Buffer::new();
                            stringify_document_with_indent(item, &mut key_buf, 0)?;
                            let key_str = key_buf.to_string();
                            destination
                                .add_bytes(&format!("{indent_str}{key_str}: null{CHAR_NEWLINE}"));
                        }
                    }
                }
            }
        }

        Node::Mapping(pairs) => {
            for (key_node, value) in pairs {
                let key_str = match key_node {
                    Node::Number(Numeric::Float(f)) => format!("\"{}\"", f),
                    Node::Number(Numeric::Integer(i)) => format!("{}", i),
                    _ => {
                        let mut key_buf = crate::io::destinations::buffer::Buffer::new();
                        stringify_document_with_indent(key_node, &mut key_buf, 0)?;
                        key_buf.to_string()
                    }
                };

                destination.add_bytes(&format!("{indent_str}{key_str}{CHAR_COLON}{CHAR_SPACE}"));

                match value {
                    Node::Array(_) | Node::Mapping(_) | Node::Set(_) => {
                        destination.add_bytes(&CHAR_NEWLINE.to_string());
                        stringify_document_with_indent(value, destination, indent + 1)?;
                    }
                    Node::Str(_, QuoteType::Unquoted, BlockStyle::Literal) => {
                        stringify_document_with_indent(value, destination, 0)?;
                    }
                    _ => {
                        stringify_document_with_indent(value, destination, 0)?;
                        destination.add_bytes(&CHAR_NEWLINE.to_string());
                    }
                }
            }
        }
        Node::Document(nodes) => {
            for node in nodes {
                stringify_document_with_indent(node, destination, indent)?;
            }
        }
        Node::Anchored(inner, name) => {
            destination.add_bytes(&format!("{CHAR_AMPERSAND}{name}{CHAR_SPACE}"));
            stringify_document_with_indent(inner, destination, indent)?;
        }
        Node::Tagged(inner, tag) => {
            destination.add_bytes(&format!("{indent_str}{tag}{CHAR_SPACE}"));
            stringify_document_with_indent(inner, destination, indent)?;
        }
        Node::Alias(name) => {
            destination.add_bytes(&format!("{CHAR_ASTERISK}{name}"));
        }
        _ => {
            return Err(crate::error::messages::ERR_UNSUPPORTED_NODE_TYPE.to_string());
        }
    }
    Ok(())
}

/// Determines if a node represents blank content that should be omitted.
///
/// Checks if a node is considered blank for stringification purposes,
/// such as None nodes, empty arrays, or empty strings.
///
/// # Arguments
///
/// * `node` - The Node to check
///
/// # Returns
///
/// true if the node is considered blank, false otherwise
fn node_is_blank(node: &Node) -> bool {
    match node {
        Node::None => true,
        Node::Comment(_) => true,
        Node::Str(s, _, _) => s.is_empty(),
        Node::Tagged(inner, _tag) => node_is_blank(inner),
        Node::Array(items) => items.iter().all(node_is_blank),
        Node::Set(items) => !items.is_empty() && items.iter().all(node_is_blank),
        Node::Mapping(pairs) => pairs.is_empty(),
        Node::Document(nodes) => nodes.iter().all(node_is_blank),
        _ => false,
    }
}

/// Stringifies a single YAML document to the destination.
///
/// Converts a document node to its YAML string representation,
/// starting with zero indentation. This is typically used for
/// individual documents within a multi-document stream.
///
/// # Arguments
///
/// * `node` - The Document Node to stringify
/// * `destination` - The output destination for the YAML content
///
/// # Returns
///
/// Result indicating success or an error string
pub fn stringify_document(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_document_with_indent(node, destination, 0)
}

/// Main entry point for stringifying YAML nodes to their text representation.
///
/// Converts any YAML node structure (documents, individual nodes) to
/// properly formatted YAML text. Handles multi-document streams by
/// adding appropriate document separators.
///
/// # Arguments
///
/// * `node` - The root Node to stringify
/// * `destination` - The output destination for the YAML content
///
/// # Returns
///
/// Result indicating success or an error string
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Documents(docs) => {
            for doc in docs {
                if let Node::Document(nodes) = doc {
                    if nodes.iter().all(node_is_blank) {
                        destination.add_bytes("---\n");
                        continue;
                    }
                }

                if let Node::Document(nodes) = doc {
                    if nodes.len() == 1 {
                        if let Node::Str(s, QuoteType::Unquoted, BlockStyle::Literal) = &nodes[0] {
                            let s = normalize_newlines(s);
                            destination
                                .add_bytes(&format!("--- {STR_LITERAL_BLOCK}{CHAR_NEWLINE}"));
                            for line in s.split(CHAR_NEWLINE) {
                                destination.add_bytes(&format!("{line}{CHAR_NEWLINE}"));
                            }
                            destination.add_bytes(&format!("...{CHAR_NEWLINE}"));
                            continue;
                        }
                    }
                }

                destination.add_bytes("---\n");
                stringify_document(doc, destination)?;
                destination.add_bytes("...\n");
            }
        }
        _ => {
            stringify_document(node, destination)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;
    use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};

    #[test]
    fn test_escape_double_basic() {
        assert_eq!(escape_double("a\"b"), "a\\\"b");

        assert_eq!(escape_double("\\n"), "\\n");

        assert_eq!(escape_double("\u{0001}"), "\\u0001");
    }

    #[test]
    fn test_escape_single_basic() {
        assert_eq!(escape_single("a'b"), "a''b");
        assert_eq!(escape_single("noquote"), "noquote");
    }

    #[test]
    fn test_normalize_newlines_removes_cr() {
        assert_eq!(normalize_newlines("line1\r\nline2\r"), "line1\nline2");
    }

    #[test]
    fn test_stringify_integer_sequence() {
        let docs = Node::Documents(vec![Node::Document(vec![Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
        ])])]);

        let mut buf = Buffer::new();
        stringify(&docs, &mut buf).expect("stringify failed");
        assert_eq!(buf.to_string(), "---\n- 1\n- 2\n- 3\n...\n");
    }

    #[test]
    fn test_stringify_mapping_simple() {
        let mapping = Node::Documents(vec![Node::Document(vec![Node::Mapping(vec![(
            Node::from("key"),
            Node::from("value"),
        )])])]);

        let mut buf = Buffer::new();
        stringify(&mapping, &mut buf).expect("stringify failed");
        assert_eq!(buf.to_string(), "---\nkey: value\n...\n");
    }

    #[test]
    fn test_stringify_single_line_literal_document_emits_pipe() {
        let lit = Node::Documents(vec![Node::Document(vec![Node::Str(
            "line1\nline2".to_string(),
            QuoteType::Unquoted,
            BlockStyle::Literal,
        )])]);

        let mut buf = Buffer::new();
        stringify(&lit, &mut buf).expect("stringify failed");
        assert_eq!(buf.to_string(), "--- |\nline1\nline2\n...\n");
    }

    #[test]
    fn test_stringify_set_simple() {
        let set_doc = Node::Documents(vec![Node::Document(vec![Node::Set(vec![
            Node::from("item1"),
            Node::from("item2"),
            Node::from("item3"),
        ])])]);

        let mut buf = Buffer::new();
        stringify(&set_doc, &mut buf).expect("stringify failed");
        assert_eq!(
            buf.to_string(),
            "---\n!!set\nitem1: null\nitem2: null\nitem3: null\n...\n"
        );
    }

    #[test]
    fn test_stringify_set_empty() {
        let set_doc = Node::Documents(vec![Node::Document(vec![Node::Set(vec![])])]);

        let mut buf = Buffer::new();
        stringify(&set_doc, &mut buf).expect("stringify failed");
        assert_eq!(buf.to_string(), "---\n!!set {}\n...\n");
    }

    #[test]
    fn test_stringify_set_with_numbers() {
        let set_doc = Node::Documents(vec![Node::Document(vec![Node::Set(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
        ])])]);

        let mut buf = Buffer::new();
        stringify(&set_doc, &mut buf).expect("stringify failed");
        assert_eq!(
            buf.to_string(),
            "---\n!!set\n1: null\n2: null\n3: null\n...\n"
        );
    }
}
