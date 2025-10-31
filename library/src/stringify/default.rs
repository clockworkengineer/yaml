use crate::constants::*;
use crate::io::traits::IDestination;
use crate::nodes::node::*;

// Escape string for double-quoted YAML scalars.
fn escape_double(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut iter = s.chars().peekable();
    while let Some(c) = iter.next() {
        match c {
            // Preserve literal newlines to support multi-line flow scalars
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
                // If this is a known YAML escape (n, r, t, b, x), emit as-is
                if let Some(&next) = iter.peek() {
                    match next {
                        'n' | 'r' | 't' | 'b' | 'x' => {
                            out.push(CHAR_BACKSLASH);
                            out.push(next);
                            iter.next(); // consume the peeked char
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

// Escape string for single-quoted YAML scalars by doubling single quotes.
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

// Normalize newlines by removing CR characters so inputs read from files
// with CRLF line endings don't emit stray '\r' characters when stringified.
fn normalize_newlines(s: &str) -> String {
    s.replace(CHAR_CARRIAGE_RETURN, "")
}

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
            // Normalize CRLF -> LF by removing CR so file-based input using
            // Windows line endings doesn't leak '\r' into the output.
            let s = normalize_newlines(s);
            match qt {
                QuoteType::Double => {
                    // escape common sequences for double-quoted output
                    destination.add_bytes(&format!(
                        "{}{}{}{}",
                        indent_str,
                        CHAR_DOUBLE_QUOTE,
                        escape_double(&s),
                        CHAR_DOUBLE_QUOTE
                    ))
                }
                QuoteType::Single => {
                    // Prefer double quotes only for single-line content that contains
                    // a single quote or backslash. Preserve single quotes for multiline
                    // scalars to match expected output semantics.
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
                        // In single-quoted YAML scalars, single quotes are represented by doubling them
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
                    // Emit literal block scalars '|' when content is multiline OR when style is explicitly Literal.
                    if s.contains(CHAR_NEWLINE) || matches!(style, BlockStyle::Literal) {
                        // For literal style, emit lines exactly as stored by the parser
                        // to avoid double-indenting when the AST already includes leading
                        // spaces. For other multiline unquoted scalars, indent by one.
                        let is_literal = matches!(style, BlockStyle::Literal);
                        // Determine if the content already contains leading indentation on all
                        // non-empty lines. If so, do not add additional emitter indentation to
                        // avoid doubling spaces. Otherwise indent by one level.
                        let lines: Vec<&str> = s.split(CHAR_NEWLINE).collect();
                        let needs_indent = if is_literal {
                            lines.iter().any(|l| !l.is_empty() && !l.starts_with(' '))
                        } else {
                            // non-literal multiline unquoted always needs indenting
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
                            // Single-line literal: emit as-is
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
            // Normalize comments as well to avoid CR characters from file sources
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
                        // Serialize mapping into a temporary buffer at child
                        // indent and strip that leading indent once so the
                        // first line appears after the "-". Later lines
                        // remain indented.
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
                        // Serialize a nested sequence into a temporary buffer at
                        // child indent and strip the leading child indent once
                        // so the first inner item follows the outer "-".
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

        Node::Mapping(pairs) => {
            // Mapping keys are Nodes; stringify each key Node into a temporary buffer
            for (key_node, value) in pairs {
                // If the key is a float number, emit it as a quoted string so
                // mapping keys like 0.25 become "0.25". Integers remain unquoted.
                let key_str = match key_node {
                    Node::Number(Numeric::Float(f)) => format!("\"{}\"", f),
                    Node::Number(Numeric::Integer(i)) => format!("{}", i),
                    _ => {
                        // Use a temporary buffer to stringify the key Node
                        let mut key_buf = crate::io::destinations::buffer::Buffer::new();
                        stringify_document_with_indent(key_node, &mut key_buf, 0)?;
                        key_buf.to_string()
                    }
                };

                destination.add_bytes(&format!("{indent_str}{key_str}{CHAR_COLON}{CHAR_SPACE}"));

                match value {
                    Node::Array(_) | Node::Mapping(_) => {
                        destination.add_bytes(&CHAR_NEWLINE.to_string());
                        stringify_document_with_indent(value, destination, indent + 1)?;
                    }
                    Node::Str(_, QuoteType::Unquoted, BlockStyle::Literal) => {
                        // Literal block already emits its own trailing newline lines; don't add another
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
            // Emit an anchor before the node
            destination.add_bytes(&format!("{CHAR_AMPERSAND}{name}{CHAR_SPACE}"));
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

// Helper to determine whether a node is blank (used when emitting documents)
fn node_is_blank(node: &Node) -> bool {
    match node {
        Node::None => true,
        Node::Comment(_) => true,
        Node::Str(s, _, _) => s.is_empty(),
        Node::Array(items) => items.iter().all(node_is_blank),
        Node::Mapping(pairs) => pairs.is_empty(),
        Node::Document(nodes) => nodes.iter().all(node_is_blank),
        _ => false,
    }
}

pub fn stringify_document(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_document_with_indent(node, destination, 0)
}

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Documents(docs) => {
            // Helper to determine whether a node contains any meaningful content
            // use module-level `node_is_blank`

            for doc in docs {
                // Emit all documents, including empty ones, to preserve explicit document boundaries
                if let Node::Document(nodes) = doc {
                    // If this document is empty, emit only the start marker and continue
                    if nodes.iter().all(node_is_blank) {
                        destination.add_bytes("---\n");
                        continue;
                    }
                }

                // Special-case: a document that is a single literal block scalar should emit
                // the '|' on the same line as the '---' per test expectations.
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
        // double-quote must be escaped
        assert_eq!(escape_double("a\"b"), "a\\\"b");
        // known escape sequences remain as backslash+char when preceded by a backslash
        assert_eq!(escape_double("\\n"), "\\n");
        // control character should be emitted as a unicode escape
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
}
