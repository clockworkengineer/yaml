use crate::io::traits::IDestination;
use crate::nodes::node::*;

fn stringify_document_with_indent(
    node: &Node,
    destination: &mut dyn IDestination,
    indent: usize,
) -> Result<(), String> {
    let indent_str = "  ".repeat(indent);
    match node {
        Node::None => destination.add_bytes(&format!("{}null", indent_str)),
        Node::Boolean(b) => destination.add_bytes(&format!("{}{}", indent_str, b)),
        Node::Str(s, qt) => match qt {
            QuoteType::Double => {
                // escape common sequences for double-quoted output
                fn escape_double(s: &str) -> String {
                    let mut out = String::with_capacity(s.len());
                    for c in s.chars() {
                        match c {
                            '\n' => out.push_str("\\n"),
                            '\r' => out.push_str("\\r"),
                            '\t' => out.push_str("\\t"),
                            '\\' => out.push_str("\\\\"),
                            '"' => out.push_str("\\\""),
                            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                                out.push_str(&format!("\\u{:04x}", c as u32));
                            }
                            other => out.push(other),
                        }
                    }
                    out
                }
                destination.add_bytes(&format!("{}\"{}\"", indent_str, escape_double(s)))
            }
            QuoteType::Single => destination.add_bytes(&format!("{}'{}'", indent_str, s)),
            QuoteType::Unquoted => destination.add_bytes(&format!("{}{}", indent_str, s)),
        },
        Node::Comment(c) => destination.add_bytes(&format!("{}# {}", indent_str, c)),
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&format!("{}{}", indent_str, i)),
            Numeric::Float(f) => destination.add_bytes(&format!("{}{}", indent_str, f)),
            _ => destination.add_bytes(&format!("{}{:?}", indent_str, num)),
        },
        Node::Array(items) => {
            for item in items {
                destination.add_bytes(&format!("{}- ", indent_str));
                match item {
                    Node::Mapping(_) => {
                        // Serialize mapping into a temporary buffer at child
                        // indent and strip that leading indent once so the
                        // first line appears after the "- ". Subsequent lines
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
                        // Serialize nested sequence into a temporary buffer at
                        // child indent and strip the leading child indent once
                        // so the first inner item follows the outer "- ".
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
                        destination.add_bytes("\n");
                    }
                }
            }
        }

        Node::Mapping(pairs) => {
            // Mapping keys are Nodes; stringify each key Node into a temporary buffer
            for (key_node, value) in pairs {
                // Use a temporary buffer to stringify the key Node
                let mut key_buf = crate::io::destinations::buffer::Buffer::new();
                stringify_document_with_indent(key_node, &mut key_buf, 0)?;
                let key_str = key_buf.to_string();

                destination.add_bytes(&format!("{}{}: ", indent_str, key_str));

                match value {
                    Node::Array(_) | Node::Mapping(_) => {
                        destination.add_bytes("\n");
                        stringify_document_with_indent(value, destination, indent + 1)?;
                    }
                    _ => {
                        stringify_document_with_indent(value, destination, 0)?;
                        destination.add_bytes("\n");
                    }
                }
            }
        }
        Node::Document(nodes) => {
            for node in nodes {
                stringify_document_with_indent(node, destination, indent)?;
            }
        }
        _ => {
            return Err("Unsupported node type".to_string());
        }
    }
    Ok(())
}

pub fn stringify_document(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_document_with_indent(node, destination, 0)
}

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Documents(docs) => {
            for doc in docs {
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
    use crate::{BufferSource, parse};

    #[test]
    fn test_stringify_none() {
        let mut dest = Buffer::new();
        stringify(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_stringify_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("test".to_string(), QuoteType::Double), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"test\"");
    }

    #[test]
    fn test_stringify_comment() {
        let mut dest = Buffer::new();
        stringify(&Node::Comment("test".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "# test");
    }

    #[test]
    fn test_stringify_numbers() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "42");

        dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(3.14)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "3.14");
    }

    #[test]
    fn test_stringify_array() {
        let mut dest = Buffer::new();
        let arr = vec![
            Node::Number(Numeric::Integer(1)),
            Node::Str("test".to_string(), QuoteType::Double),
        ];
        stringify(&Node::Array(arr), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "- 1\n- \"test\"\n");
    }

    #[test]
    fn test_stringify_mapping() {
        let mut dest = Buffer::new();
        let mapping = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Double),
            Node::Str("value".to_string(), QuoteType::Double),
        )]);
        stringify(&mapping, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"key\": \"value\"\n");
    }

    #[test]
    fn test_stringify_documents() {
        let mut dest = Buffer::new();
        let docs = vec![
            Node::Str("doc1".to_string(), QuoteType::Double),
            Node::Str("doc2".to_string(), QuoteType::Double),
        ];
        stringify(&Node::Documents(docs), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n\"doc1\"...\n---\n\"doc2\"...\n");
    }

    #[test]
    fn test_stringify_integer_sequence() {
        let mut dest = Buffer::new();
        let mut source = BufferSource::new("---\n- 1\n- 2\n- 3\n...\n".as_bytes());
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n- 1\n- 2\n- 3\n...\n");
    }
    #[test]
    fn test_stringify_sequence_with_nested_mapping() {
        let mut dest = Buffer::new();
        let mut source = BufferSource::new("---\n- \n  name: Mark Joseph\n  hr: 87\n  avg: 0.278\n- \n  name: James Stephen\n  hr: 63\n  avg: 0.288\n...\n".as_bytes());
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\n- name: Mark Joseph\n  hr: 87\n  avg: 0.278\n- name: James Stephen\n  hr: 63\n  avg: 0.288\n...\n"
        );
    }
    #[test]
    fn test_stringify_sequence_with_nested_sequence() {
        let mut dest = Buffer::new();
        let mut source = BufferSource::new("- [Sammy Sosa, 63, 0.288]".as_bytes());
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\n- - Sammy Sosa\n  - 63\n  - 0.288\n...\n"
        );
    }

}
