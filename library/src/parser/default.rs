//! YAML parser implementation that converts YAML text into Node structures
//! Provides functions for parsing different YAML data types including mappings,
//! sequences, strings, numbers, boolean and null values.

use crate::nodes::node::Node;
use crate::nodes::node::Numeric;
use std::collections::HashMap;
use crate::io::traits::ISource;
use crate::nodes::node::Node::Document;

fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if source.is_whitespace(c) {
            source.next();
        } else {
            break;
        }
    }
}

fn skip_until_newline(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c == '\n' {
            source.next();
            break;
        }
        source.next();
    }
}
fn parse_comment(source: &mut dyn ISource) -> String {
    source.next(); // Skip the '#' character
    let mut comment = String::new();
    while let Some(c) = source.current() {
        if c == '\n' { break; }
        comment.push(c);
        source.next();
    }
    comment.trim().to_string()
}

fn parse_scalar(value: &str) -> Node {
    // Check if the value is a comment (starts with #)
    if value.starts_with('#') {
        Node::Comment(value[1..].trim().to_string())
    } else if value == "null" || value == "~" {
        Node::None
    } else if value == "true" {
        Node::Boolean(true)
    } else if value == "false" {
        Node::Boolean(false)
    } else if let Ok(i) = value.parse::<i64>() {
        Node::Number(Numeric::Integer(i))
    } else if let Ok(f) = value.parse::<f64>() {
        Node::Number(Numeric::Float(f))
    } else {
        Node::Str(value.to_string())
    }
}

fn parse_sequence(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    let mut items = Vec::new();
    while let Some(c) = source.current() {
        if c == '#' {
            items.push(Node::Comment(parse_comment(source)));
            continue;
        }

        let current_indent = source.get_current_indent_level();
        if current_indent < indent_level {
            break;
        }

        if c == '-' {
            source.next(); // Skip the dash
            skip_whitespace(source);

            if let Some(next_c) = source.current() {
                if next_c == '-' {
                    // Check for nested sequence
                    let nested_indent = source.get_current_indent_level();
                    items.push(parse_sequence(source, nested_indent)?);
                    continue;
                } else {
                    // Parse scalar value
                    let mut value = String::new();
                    while let Some(c) = source.current() {
                        if c == '\n' || c == '#' { break; }
                        value.push(c);
                        source.next();
                    }
                    if !value.trim().is_empty() {
                        items.push(parse_scalar(value.trim()));
                    }
                }
            }
        } else if !c.is_whitespace() {
            return Err(format!("Expected sequence item starting with '-', got '{}'", c));
        }

        skip_until_newline(source);
        skip_whitespace(source);

    }
    Ok(Node::Array(items))
}

fn parse_mapping(source: &mut dyn ISource, _indent_level:usize) -> Result<Node, String> {
    let mut map = HashMap::new();
    while let Some(c) = source.current() {
        if peek_ahead_for_document_start(source) {
            return Ok(Node::Dictionary(map));
        } else if c == '#' {
            parse_comment(source);
        } else if c.is_alphanumeric() {
            let mut key = String::new();
            while let Some(c) = source.current() {
                if c == ':' { break; }
                key.push(c);
                source.next();
            }
            source.next(); // Skip ':'
            skip_whitespace(source);

            let mut value = String::new();
            while let Some(c) = source.current() {
                if c == '\n' || c == '#' { break; }
                value.push(c);
                source.next();
            }

            map.insert(key.trim().to_string(), parse_scalar(value.trim()));
        }
        source.next();
    }
    Ok(Node::Dictionary(map))
}

fn peek_ahead_for_document_start(source: &mut dyn ISource) -> bool {
    if source.current() != Some('-') {
        return false;
    }
    source.next();
    if source.current() != Some('-') {
        source.backup();
        return false;
    }
    source.next();
    if source.current() != Some('-') {
        source.backup();
        source.backup();
        return false;
    }
    source.backup();
    source.backup();
    true
}

pub fn parse_documents(source: &mut dyn ISource, indent_level:usize) -> Result<Node, String> {
    skip_whitespace(source);

    let mut documents = Vec::new();
    let mut current_document = None;

    while let Some(c) = source.current() {
        match c {
            '-' if peek_ahead_for_document_start(source) => {
                if let Some(doc) = current_document.take() {
                    documents.push(doc);
                }
                // Skip the document separator
                source.next();
                source.next();
                source.next();
                skip_whitespace(source);
                return Ok(Document(documents))
            }
            '-' => {
                let indent_level = source.get_current_indent_level();
                current_document = Some(parse_sequence(source, indent_level)?);
            }
            '#' => {
                let comment =parse_comment(source);
                if let Some(doc) = current_document {
                    documents.push(doc);
                }
                current_document = Some(Node::Comment(comment.trim().to_string()));
            }
            c if c.is_alphanumeric() => {
                current_document = Some(parse_mapping(source,indent_level)?);
            }
            c if c.is_whitespace() => {
                source.next();
            }
            c => return Err(format!("Unexpected character: {}", c))
        }
        if let Some(doc) = &current_document {
            documents.push(doc.clone());
            current_document = None;
        }
    }

    Ok(Document(documents))

}
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    let mut docs: Vec<Node> = Vec::new();
    while source.more() {
        let document = parse_documents(source, 0);
        match document {
            Ok(doc) => {
                docs.push(doc.into());
            }
            Err(err) => {
                return Err(err);
            }
        };
    }
    Ok(Node::Documents(docs))
}

pub fn get_number_of_documents(documents: &Node) -> Result<usize, String> {
    match documents {
        Node::Documents(docs) => Ok(docs.len()),
        _ => Err("Expected Documents node".to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_parse_scalar() {
        assert_eq!(parse_scalar("null"), Node::None);
        assert_eq!(parse_scalar("~"), Node::None);
        assert_eq!(parse_scalar("true"), Node::Boolean(true));
        assert_eq!(parse_scalar("false"), Node::Boolean(false));
        assert_eq!(parse_scalar("42"), Node::Number(Numeric::Integer(42)));
        assert_eq!(parse_scalar("3.14"), Node::Number(Numeric::Float(3.14)));
        assert_eq!(parse_scalar("hello"), Node::Str("hello".to_string()));
        assert_eq!(parse_scalar("#comment"), Node::Comment("comment".to_string()));
    }

    #[test]
    fn test_parse_sequence() {
        let mut source = Buffer::new(b"- 1\n- 2\n- 3");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3))
        ])])]));
    }

    #[test]
    fn test_parse_sequence_with_comments() {
        let mut source = Buffer::new(b"- 1\n# Comment 1\n- 2\n# Comment 2");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Comment("Comment 1".to_string()),
            Node::Number(Numeric::Integer(2)),
            Node::Comment("Comment 2".to_string())
        ])])]));
    }

    #[test]
    fn test_parse_mapping() {
        let mut source = Buffer::new(b"key1: value1\nkey2: 42");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), Node::Str("value1".to_string()));
        expected.insert("key2".to_string(), Node::Number(Numeric::Integer(42)));
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(expected)])]));
    }

    #[test]
    fn test_parse_empty() {
        let mut source = Buffer::new(b"");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![]));
    }

    #[test]
    fn test_parse_invalid_char() {
        let mut source = Buffer::new(b"@invalid");
        let result = parse(&mut source);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unexpected character: @");
    }

    #[test]
    fn test_parse_comment_only() {
        let mut source = Buffer::new(b"# Just a comment");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Comment("Just a comment".to_string())])]));
    }

    #[test]
    fn test_parse_multi_document() {
        let mut source = Buffer::new(b"key1: value1\n---\nkey2: value2\n---\nkey3: value3\nkey4: value4\n");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert("key1".to_string(), Node::Str("value1".to_string()));
        let mut doc2 = HashMap::new();
        doc2.insert("key2".to_string(), Node::Str("value2".to_string()));
        let mut doc3 = HashMap::new();
        doc3.insert("key3".to_string(), Node::Str("value3".to_string()));
        doc3.insert("key4".to_string(), Node::Str("value4".to_string()));
        assert_eq!(result, Node::Documents(vec![
            Document(vec![Node::Dictionary(doc1)]),
            Document(vec![Node::Dictionary(doc2)]),
            Document(vec![Node::Dictionary(doc3)])
        ]));
    }

    #[test]
    fn test_parse_header_comments() {
        let mut source = Buffer::new(b"# Header comment 1\n# Header comment 2\n# Header comment 3\nkey: value\n");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![
            Document(vec![
                Node::Comment("Header comment 1".to_string()),
                Node::Comment("Header comment 2".to_string()),
                Node::Comment("Header comment 3".to_string()),
                Node::Dictionary({
                    let mut map = HashMap::new();
                    map.insert("key".to_string(), Node::Str("value".to_string()));
                    map
                })
            ])
        ]));
    }

    #[test]
    fn test_parse_nested_sequence() {
        let mut source = Buffer::new(b"- item1\n- - nested1\n  - nested2\n- item2");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Str("item1".to_string()),
            Node::Array(vec![
                Node::Str("nested1".to_string()),
                Node::Str("nested2".to_string())
            ]),
            Node::Str("item2".to_string())
        ])])]));
    }

    #[test]
    fn test_get_number_of_documents() {
        let mut source = Buffer::new(b"doc1: value1\n---\ndoc2: value2\n---\ndoc3: value3");
        let result = parse(&mut source).unwrap();
        assert_eq!(get_number_of_documents(&result).unwrap(), 3);

        // Test error case with non-Documents node
        let non_docs_node = Node::Str("test".to_string());
        assert!(get_number_of_documents(&non_docs_node).is_err());
    }
    #[test]
    fn test_parse_mapping_with_comments() {
        let mut source = Buffer::new(b"key1: value1\n# Comment 1\nkey2: 42\n# Comment 2");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), Node::Str("value1".to_string()));
        expected.insert("key2".to_string(), Node::Number(Numeric::Integer(42)));
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(expected)])]));
    }

}





