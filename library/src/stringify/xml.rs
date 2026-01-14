//! Module: stringify/xml.rs

use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::format::node_to_key_like_string;
use crate::utils::escape::escape_for_xml;

fn escape_xml_string(s: &str) -> String {
    escape_for_xml(s)
}

fn write_xml_text(s: &str, destination: &mut dyn IDestination) {
    destination.add_bytes(&escape_xml_string(s));
}

fn sanitize_tag(name: &str) -> String {
    // Simplified XML name sanitizer (ASCII subset):
    // - First character must be letter, '_' or ':'
    // - Subsequent characters may be letter, digit, '.', '-', '_' or ':'
    // Any invalid character is replaced with '_'. Empty or all-invalid names become "item".
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "item".to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    for (i, c) in trimmed.chars().enumerate() {
        let valid = if i == 0 {
            matches!(c, 'a'..='z' | 'A'..='Z' | '_' | ':')
        } else {
            matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | ':')
        };
        if valid {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "item".to_string()
    } else if out.chars().next().unwrap().is_ascii_digit() {
        // ensure name doesn't start with digit
        format!("n{}", out)
    } else {
        out
    }
}

fn stringify_node(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => {
            // represent as empty text
        }
        Node::Boolean(b) => destination.add_bytes(if *b { "true" } else { "false" }),
        Node::Str(s, _, _) => write_xml_text(s, destination),
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&i.to_string()),
            Numeric::Float(f) => destination.add_bytes(&f.to_string()),
            Numeric::UInteger(u) => destination.add_bytes(&u.to_string()),
            Numeric::Byte(b) => destination.add_bytes(&b.to_string()),
            Numeric::Int32(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt32(u) => destination.add_bytes(&u.to_string()),
            Numeric::Int16(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt16(u) => destination.add_bytes(&u.to_string()),
            Numeric::Int8(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt8(u) => destination.add_bytes(&u.to_string()),
        },
        Node::Array(items) => {
            for item in items.iter() {
                destination.add_bytes("<item>");
                stringify_node(item, destination)?;
                destination.add_bytes("</item>");
            }
        }
        Node::Set(items) => {
            // Represent sets as XML elements with set attribute
            for item in items.iter() {
                destination.add_bytes("<item type=\"set\">");
                stringify_node(item, destination)?;
                destination.add_bytes("</item>");
            }
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs.iter() {
                let key_str = node_to_key_like_string(k);
                let tag = sanitize_tag(&key_str);
                destination.add_bytes("<");
                destination.add_bytes(&tag);

                match v {
                    Node::Mapping(_) | Node::Array(_) | Node::Document(_) | Node::Documents(_) => {
                        destination.add_bytes(">");
                        stringify_node(v, destination)?;
                        destination.add_bytes("</");
                        destination.add_bytes(&tag);
                        destination.add_bytes(">");
                    }
                    Node::None => {
                        destination.add_bytes("/>");
                    }
                    _ => {
                        destination.add_bytes(">");
                        stringify_node(v, destination)?;
                        destination.add_bytes("</");
                        destination.add_bytes(&tag);
                        destination.add_bytes(">");
                    }
                }
            }
        }
        Node::Document(nodes) => {
            if nodes.len() == 1 {
                stringify_node(&nodes[0], destination)?;
            } else {
                destination.add_bytes("<document>");
                for n in nodes.iter() {
                    destination.add_bytes("<node>");
                    stringify_node(n, destination)?;
                    destination.add_bytes("</node>");
                }
                destination.add_bytes("</document>");
            }
        }
        Node::Tagged(inner, _tag) => stringify_node(inner, destination)?,
        Node::Anchored(inner, _name) => stringify_node(inner, destination)?,
        Node::Alias(_name) => {
            // no representation for alias; emit empty
        }
        Node::Documents(docs) => {
            destination.add_bytes("<documents>");
            for d in docs.iter() {
                destination.add_bytes("<document>");
                stringify_node(d, destination)?;
                destination.add_bytes("</document>");
            }
            destination.add_bytes("</documents>");
        }
        Node::Comment(c) => {
            // emit as XML comment
            destination.add_bytes("<!--");
            destination.add_bytes(c);
            destination.add_bytes("-->");
        }
    }
    Ok(())
}

/// stringify
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_node(node, destination)
}

/// Pretty-print XML (delegates to same behaviour for now)
pub fn stringify_pretty(
    node: &Node,
    destination: &mut dyn IDestination,
    spaces_per_indent: usize,
) -> Result<(), String> {
    fn write_indent(dest: &mut dyn IDestination, level: usize, spaces: usize) {
        if spaces == 0 || level == 0 {
            return;
        }
        let s = " ".repeat(spaces * level);
        dest.add_bytes(&s);
    }

    fn helper(
        node: &Node,
        dest: &mut dyn IDestination,
        spaces: usize,
        level: usize,
    ) -> Result<(), String> {
        match node {
            Node::None => {
                // nothing
            }
            Node::Boolean(b) => dest.add_bytes(if *b { "true" } else { "false" }),
            Node::Str(s, _, _) => write_xml_text(s, dest),
            Node::Number(num) => match num {
                Numeric::Integer(i) => dest.add_bytes(&i.to_string()),
                Numeric::Float(f) => dest.add_bytes(&f.to_string()),
                Numeric::UInteger(u) => dest.add_bytes(&u.to_string()),
                Numeric::Byte(b) => dest.add_bytes(&b.to_string()),
                Numeric::Int32(i) => dest.add_bytes(&i.to_string()),
                Numeric::UInt32(u) => dest.add_bytes(&u.to_string()),
                Numeric::Int16(i) => dest.add_bytes(&i.to_string()),
                Numeric::UInt16(u) => dest.add_bytes(&u.to_string()),
                Numeric::Int8(i) => dest.add_bytes(&i.to_string()),
                Numeric::UInt8(u) => dest.add_bytes(&u.to_string()),
            },
            Node::Array(items) => {
                for item in items.iter() {
                    write_indent(dest, level, spaces);
                    dest.add_bytes("<item>");
                    // scalar or complex?
                    match item {
                        Node::Mapping(_)
                        | Node::Array(_)
                        | Node::Set(_)
                        | Node::Document(_)
                        | Node::Documents(_) => {
                            dest.add_bytes("\n");
                            helper(item, dest, spaces, level + 1)?;
                            dest.add_bytes("\n");
                            write_indent(dest, level, spaces);
                            dest.add_bytes("</item>");
                        }
                        _ => {
                            helper(item, dest, spaces, 0)?;
                            dest.add_bytes("</item>");
                        }
                    }
                    dest.add_bytes("\n");
                }
            }
            Node::Set(items) => {
                for item in items.iter() {
                    write_indent(dest, level, spaces);
                    dest.add_bytes("<item type=\"set\">");
                    // scalar or complex?
                    match item {
                        Node::Mapping(_)
                        | Node::Array(_)
                        | Node::Set(_)
                        | Node::Document(_)
                        | Node::Documents(_) => {
                            dest.add_bytes("\n");
                            helper(item, dest, spaces, level + 1)?;
                            dest.add_bytes("\n");
                            write_indent(dest, level, spaces);
                            dest.add_bytes("</item>");
                        }
                        _ => {
                            helper(item, dest, spaces, 0)?;
                            dest.add_bytes("</item>");
                        }
                    }
                    dest.add_bytes("\n");
                }
            }
            Node::Mapping(pairs) => {
                for (k, v) in pairs.iter() {
                    let key_str = node_to_key_like_string(k);
                    let tag = sanitize_tag(&key_str);
                    write_indent(dest, level, spaces);
                    dest.add_bytes("<");
                    dest.add_bytes(&tag);

                    match v {
                        Node::Mapping(_)
                        | Node::Array(_)
                        | Node::Set(_)
                        | Node::Document(_)
                        | Node::Documents(_) => {
                            dest.add_bytes(">");
                            dest.add_bytes("\n");
                            helper(v, dest, spaces, level + 1)?;
                            // child helper is expected to emit a trailing newline for its block content
                            write_indent(dest, level, spaces);
                            dest.add_bytes("</");
                            dest.add_bytes(&tag);
                            dest.add_bytes(">");
                        }
                        Node::None => {
                            dest.add_bytes(" />");
                        }
                        _ => {
                            dest.add_bytes(">");
                            helper(v, dest, spaces, 0)?;
                            dest.add_bytes("</");
                            dest.add_bytes(&tag);
                            dest.add_bytes(">");
                        }
                    }
                    dest.add_bytes("\n");
                }
            }
            Node::Document(nodes) => {
                if nodes.len() == 1 {
                    helper(&nodes[0], dest, spaces, level)?;
                } else {
                    write_indent(dest, level, spaces);
                    dest.add_bytes("<document>\n");
                    for n in nodes.iter() {
                        helper(n, dest, spaces, level + 1)?;
                        dest.add_bytes("\n");
                    }
                    write_indent(dest, level, spaces);
                    dest.add_bytes("</document>");
                }
            }
            Node::Tagged(inner, _tag) => helper(inner, dest, spaces, level)?,
            Node::Anchored(inner, _name) => helper(inner, dest, spaces, level)?,
            Node::Alias(_name) => {
                // nothing
            }
            Node::Documents(docs) => {
                write_indent(dest, level, spaces);
                dest.add_bytes("<documents>\n");
                for d in docs.iter() {
                    write_indent(dest, level + 1, spaces);
                    dest.add_bytes("<document>\n");
                    helper(d, dest, spaces, level + 2)?;
                    write_indent(dest, level + 1, spaces);
                    dest.add_bytes("</document>\n");
                }
                write_indent(dest, level, spaces);
                dest.add_bytes("</documents>");
            }
            Node::Comment(c) => {
                write_indent(dest, level, spaces);
                dest.add_bytes("<!--");
                dest.add_bytes(c);
                dest.add_bytes("-->");
            }
        }
        Ok(())
    }

    helper(node, destination, spaces_per_indent, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer as BufferDestination;
    use crate::nodes::node::{BlockStyle, Node, QuoteType};

    #[test]
    fn test_xml_basic() {
        let mut buf = BufferDestination::new();
        let n = Node::Str(
            "hello & <world>".to_string(),
            QuoteType::Unquoted,
            BlockStyle::None,
        );
        stringify(&n, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "hello &amp; &lt;world&gt;");

        buf.clear();
        let m = Node::Mapping(vec![(
            Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(1),
        )]);
        stringify(&m, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "<a>1</a>");
    }

    #[test]
    fn test_xml_array() {
        let mut buf = BufferDestination::new();
        let arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        stringify(&arr, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "<item>1</item><item>2</item>");
    }

    #[test]
    fn test_xml_nested_mapping() {
        let mut buf = BufferDestination::new();
        let nested = Node::Mapping(vec![(
            Node::Str("parent".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![(
                Node::Str("child".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::from(2),
            )]),
        )]);
        stringify(&nested, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "<parent><child>2</child></parent>");
    }

    #[test]
    fn test_xml_special_key() {
        let mut buf = BufferDestination::new();
        let key = "123 key".to_string();
        let m = Node::Mapping(vec![(
            Node::Str(key.clone(), QuoteType::Unquoted, BlockStyle::None),
            Node::from("value"),
        )]);
        stringify(&m, &mut buf).unwrap();
        // sanitized tag should replace invalid chars with '_' and not panic
        assert!(buf.to_string().contains("value"));
    }

    #[test]
    fn test_xml_pretty() {
        let mut buf = BufferDestination::new();
        let nested = Node::Mapping(vec![(
            Node::Str("parent".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![(
                Node::Str("child".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::from(1),
            )]),
        )]);
        stringify_pretty(&nested, &mut buf, 2).unwrap();
        let expected = "<parent>\n  <child>1</child>\n</parent>\n";
        assert_eq!(buf.to_string(), expected);
    }

    #[test]
    fn test_xml_empty_string() {
        let mut buf = BufferDestination::new();
        let n = Node::Str("".to_string(), QuoteType::Unquoted, BlockStyle::None);
        stringify(&n, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "");
    }

    #[test]
    fn test_xml_none_self_closing() {
        let mut buf = BufferDestination::new();
        let m = Node::Mapping(vec![(
            Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::None,
        )]);
        stringify(&m, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "<a/>");
    }

    #[test]
    fn test_xml_comment() {
        let mut buf = BufferDestination::new();
        let c = Node::Comment("a comment".to_string());
        stringify(&c, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "<!--a comment-->");
    }

    #[test]
    fn test_xml_documents_multiple() {
        let mut buf = BufferDestination::new();
        let docs = Node::Documents(vec![
            Node::Document(vec![Node::from(1)]),
            Node::Document(vec![Node::from(2)]),
        ]);
        stringify(&docs, &mut buf).unwrap();
        assert_eq!(
            buf.to_string(),
            "<documents><document>1</document><document>2</document></documents>"
        );
    }

    #[test]
    fn test_xml_array_of_maps() {
        let mut buf = BufferDestination::new();
        let arr = Node::Array(vec![Node::Mapping(vec![(
            Node::Str("k".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(1),
        )])]);
        stringify(&arr, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "<item><k>1</k></item>");
    }

    #[test]
    fn test_sanitize_leading_digit() {
        let mut buf = BufferDestination::new();
        let key = "123key".to_string();
        let m = Node::Mapping(vec![(
            Node::Str(key.clone(), QuoteType::Unquoted, BlockStyle::None),
            Node::from("v"),
        )]);
        stringify(&m, &mut buf).unwrap();
        let out = buf.to_string();
        // tag should not begin with ASCII digit
        let tag_start = out.find('<').map(|i| out.chars().nth(i + 1)).flatten();
        assert!(tag_start.is_some());
        assert!(!tag_start.unwrap().is_ascii_digit());
        assert!(out.contains("v"));
    }
}
