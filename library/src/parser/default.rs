//! YAML parser implementation that converts YAML text into Node structures
//! Provides functions for parsing different YAML data types including mappings,
//! sequences, strings, numbers, boolean and null values.

use crate::nodes::node::{Node, Numeric};
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

fn peek_ahead_for_document_start_end(source: &mut dyn ISource, c: char) -> bool {
    if source.current() != Some(c) {
        return false;
    }
    source.next();
    if source.current() != Some(c) {
        source.backup();
        return false;
    }
    source.next();
    if source.current() != Some(c) {
        source.backup();
        source.backup();
        return false;
    }
    source.backup();
    source.backup();
    true
}
fn peek_ahead_for_mapping_key(source: &mut dyn ISource) -> bool {
    let mut found = false;
    let mut backup_count = 0;

    while let Some(c) = source.current() {
        match c {
            ':' => {
                found = true;
                break;
            }
            '\n' => {
                break;
            }
            _ => {
                if source.more() {
                    source.next();
                    backup_count += 1;
                }
            }
        }
    }

    // Restore position
    for _ in 0..backup_count - 1 {
        source.backup();
    }

    if source.get_current_indent_level() > 0 {
        source.backup();
    }

    found
}



fn parse_mapping_key(source: &mut dyn ISource) -> Result<(String, bool), String> {
    let mut key = String::new();
    while let Some(c) = source.current() {
        if c == ':' { break; }
        key.push(c);
        source.next();
    }

    let mut newline = false;
    source.next(); // Skip ':'
    if let Some(next_char) = source.current() {
        if !next_char.is_whitespace() {
            return Err("Expected space after colon in mapping".to_string());
        }
    }
    skip_whitespace(source);
    if let Some(c) = source.current() {
        newline = c == '\n';
        if newline {
            source.next();
            skip_whitespace(source);
        }
    }

    Ok((key, newline))
}
fn parse_value(source: &mut dyn ISource) -> Result<Node, String> {
    // Support inline mapping values starting with '{'
    if source.current() == Some('{') {
        return parse_inline_mapping(source);
    }

    let mut value = String::new();
    while let Some(c) = source.current() {
        if c == '\n' || c == '#' { break; }
        value.push(c);
        source.next();
    }
    if !value.trim().is_empty() {
        Ok(parse_scalar(value.trim()))
    } else {
        Ok(Node::None)
    }
}

fn parse_inline_mapping(source: &mut dyn ISource) -> Result<Node, String> {
    // Assumes current char is '{'
    let mut map = HashMap::new();
    // consume '{'
    source.next();
    // skip whitespace
    skip_whitespace(source);

    // Handle empty mapping
    if source.current() == Some('}') {
        source.next(); // consume '}'
        return Ok(Node::Dictionary(map));
    }

    loop {
        // Parse key
        let mut key = String::new();
        while let Some(c) = source.current() {
            if c == ':' { break; }
            if c == '}' {
                // Trailing '}' without key:value
                break;
            }
            key.push(c);
            source.next();
        }
        if source.current() != Some(':') {
            return Err("Expected ':' in inline mapping".to_string());
        }
        source.next(); // consume ':'
        // value may start with space
        skip_whitespace(source);

        // Parse value
        let value_node = if source.current() == Some('{') {
            parse_inline_mapping(source)?
        } else {
            // collect until ',' or '}' or '#'
            let mut val = String::new();
            while let Some(c) = source.current() {
                match c {
                    ',' | '}' | '#' => break,
                    _ => {
                        val.push(c);
                        source.next();
                    }
                }
            }
            parse_scalar(val.trim())
        };

        map.insert(key.trim().to_string(), value_node);

        // After value, skip whitespace and optional comment (until end or before comma/})
        skip_whitespace(source);
        if source.current() == Some('#') {
            // skip comment until end of line
            skip_until_newline(source);
            // inside inline mapping, a newline should be followed by more content or end
            skip_whitespace(source);
        }

        match source.current() {
            Some(',') => {
                source.next();
                skip_whitespace(source);
                continue;
            }
            Some('}') => {
                source.next();
                break;
            }
            Some(c) => {
                return Err(format!("Unexpected character in inline mapping: {}", c));
            }
            None => return Err("Unexpected end of input in inline mapping".to_string()),
        }
    }

    Ok(Node::Dictionary(map))
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
    match value {
        v if v.starts_with('#') => Node::Comment(v[1..].trim().to_string()),
        "null" | "~" => Node::None,
        "true" => Node::Boolean(true),
        "false" => Node::Boolean(false),
        v => {
            if let Ok(i) = v.parse::<i64>() {
                Node::Number(Numeric::Integer(i))
            } else if let Ok(f) = v.parse::<f64>() {
                Node::Number(Numeric::Float(f))
            } else {
                Node::Str(v.to_string())
            }
        }
    }
}

fn parse_sequence(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    let mut items = Vec::new();
    while let Some(c) = source.current() {

        if source.get_current_indent_level() < indent_level {
            break;
        }

        match c {
            '#' => {
                items.push(Node::Comment(parse_comment(source)));
                continue;
            }
            '-'|'.' if peek_ahead_for_document_start_end(source,c) => {
                break;
            },
            '-' => {
                source.next(); // Skip the dash
                skip_whitespace(source);
                if source.current() == Some('\n') {
                    source.next();
                    skip_whitespace(source);
                }

                if let Some(next_c) = source.current() {
                    match next_c {
                        '-' => {
                            // Check for a nested sequence
                            let nested_indent = source.get_current_indent_level();
                            items.push(parse_document_contents(source, nested_indent)?);
                            continue;
                        },
                        _ => {
                            if peek_ahead_for_mapping_key(source) {
                                let nested_indent = source.get_current_indent_level();
                                items.push(parse_document_contents(source, nested_indent)?);
                                continue;
                            }
                            // Parse scalar value
                            else {
                                items.push(parse_value(source)?);
                            }
                        }
                    }
                }
            },
            _ if !c.is_whitespace() => {
                return Err(format!("Expected sequence item starting with '-', got '{}'", c));
            },
            _ => ()
        }

        skip_until_newline(source);
        skip_whitespace(source);

    }
    Ok(Node::Array(items))
}

fn parse_mapping(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    let mut map = HashMap::new();
    while let Some(c) = source.current() {
        match c {
            '-'|'.'  if peek_ahead_for_document_start_end(source,c) => {
               break;
            },
            '#' => {
                parse_comment(source);
            }
            c if c.is_alphanumeric() => {

                if  source.get_current_indent_level() < indent_level {
                    break;
                }

                let (key, newline) = parse_mapping_key(source)?;

                let next_indent = source.get_current_indent_level();
                if next_indent > indent_level && newline {
                    map.insert(key.trim().to_string(), parse_document_contents(source, next_indent)?);
                    continue;
                } else {
                    map.insert(key.trim().to_string(), parse_value(source)?);
                }
            }
            c if c.is_whitespace() => {
                source.next();
                continue;
            }
            _ => break,
        }
        skip_until_newline(source);
        skip_whitespace(source);
    }
    Ok(Node::Dictionary(map))
}


pub fn parse_document_contents(source: &mut dyn ISource, indent_level:usize) -> Result<Node, String> {
     match source.current() {
        Some('-') => {
            let indent_level = source.get_current_indent_level();
            Ok(parse_sequence(source, indent_level)?)
        }
        Some('#') => {
            let comment = parse_comment(source);
            Ok(Node::Comment(comment.trim().to_string()))
        }
        Some('{') => {
            Ok(parse_inline_mapping(source)?)
        }
        Some(c) if c.is_alphanumeric() => {
            Ok(parse_mapping(source, indent_level)?)
        }
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(source, indent_level)?)
        }
        Some(c) => Err(format!("Unexpected character: {}", c)),
        None => Err("Unexpected end of input".to_string())
    }

}
pub fn parse_document(source: &mut dyn ISource, indent_level:usize) -> Result<Node, String> {

    skip_whitespace(source);

    let mut document_nodes = Vec::new();

    while let Some(c) = source.current() {
        match c {
            '-'|'.' if peek_ahead_for_document_start_end(source,c) => {
                skip_until_newline(source);
                skip_whitespace(source);
                break;
                
            }
            _ => {
                document_nodes.push(parse_document_contents(source, indent_level)?);
            }
       }
    }

    Ok(Document(document_nodes))

}
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    let mut docs: Vec<Node> = Vec::new();
    while source.more() {
        let document = parse_document(source, 0);
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
    use crate::io::sources::file::File as FileSource;
    use std::fs;
    fn get_json_file_paths(directory: &str) -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                        if let Some(path_str) = path.to_str() {
                            paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
        paths
    }

    #[test]
    fn test_parse_json_files() {
        let files_dir = "../files";
        let json_files = get_json_file_paths(files_dir);
        for file_path in json_files {
            match FileSource::new(&file_path.to_string()) {
                Ok(mut source) => {
                    let result = parse(&mut source);
                    assert!(result.is_ok(), "Failed to parse {}: {:?}", file_path, result.err());
                },
                Err(e) => panic!("Failed to open {}: {}", file_path, e),
            }


        }
    }

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

    #[test]
    fn test_parse_nested_mapping() {
        let mut source = Buffer::new(b"outer:\n  inner1: value1\n  inner2: value2");
        let result = parse(&mut source).unwrap();

        let mut inner_map = HashMap::new();
        inner_map.insert("inner1".to_string(), Node::Str("value1".to_string()));
        inner_map.insert("inner2".to_string(), Node::Str("value2".to_string()));

        let mut outer_map = HashMap::new();
        outer_map.insert("outer".to_string(), Node::Dictionary(inner_map));

        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(outer_map)])]));
    }
    #[test]
    fn test_parse_nested_mapping_with_key_after_nested() {
        let mut source = Buffer::new(b"outer1:\n  inner1: value1\n  inner2: value2\nouter2: value3");
        let result = parse(&mut source).unwrap();

        let mut inner_map = HashMap::new();
        inner_map.insert("inner1".to_string(), Node::Str("value1".to_string()));
        inner_map.insert("inner2".to_string(), Node::Str("value2".to_string()));

        let mut outer_map = HashMap::new();
        outer_map.insert("outer1".to_string(), Node::Dictionary(inner_map));
        outer_map.insert("outer2".to_string(), Node::Str("value3".to_string()));

        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(outer_map)])]));
    }

    #[test]
    fn test_parse_mapping_with_nested_sequence() {
        let mut source = Buffer::new(b"key1:\n  - item1\n  - item2\nkey2: value2");
        let result = parse(&mut source).unwrap();

        let sequence = Node::Array(vec![
            Node::Str("item1".to_string()),
            Node::Str("item2".to_string())
        ]);

        let mut map = HashMap::new();
        map.insert("key1".to_string(), sequence);
        map.insert("key2".to_string(), Node::Str("value2".to_string()));

        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(map)])]));
    }
    #[test]
    fn test_parse_mapping_with_nested_sequence_and_comments() {
        let mut source = Buffer::new(b"key1:\n  - item1\n  - item2\n# Comment 1\nkey2: value2\n# Comment 2");
        let result = parse(&mut source).unwrap();
        let sequence = Node::Array(vec![
            Node::Str("item1".to_string()),
            Node::Str("item2".to_string())
        ]);
        let mut map = HashMap::new();
        map.insert("key1".to_string(), sequence);
        map.insert("key2".to_string(), Node::Str("value2".to_string()));
        assert_eq!(result, Node::Documents(vec![Document(vec![
            // Node::Comment("Comment 1".to_string()),
            Node::Dictionary(map),
            // Node::Comment("Comment 2".to_string())
        ])]));
    }

    #[test]
    fn test_parse_sequence_with_nested_comments() {
        let mut source = Buffer::new(b"- item1\n# Comment between items\n- item2\n# Final comment\n- item3");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Str("item1".to_string()),
            Node::Comment("Comment between items".to_string()),
            Node::Str("item2".to_string()),
            Node::Comment("Final comment".to_string()),
            Node::Str("item3".to_string())
        ])])]));
    }

    #[test]
    fn test_parse_document_end_marker() {
        let mut source = Buffer::new(b"key: value\n---");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), Node::Str("value".to_string()));
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(expected)])]));
    }

    #[test]
    fn test_parse_document_end_marker_with_trailing_content() {
        let mut source = Buffer::new(b"key: value\n---\nother: 123");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert("key".to_string(), Node::Str("value".to_string()));
        let mut doc2 = HashMap::new();
        doc2.insert("other".to_string(), Node::Number(Numeric::Integer(123)));
        assert_eq!(result, Node::Documents(vec![
            Document(vec![Node::Dictionary(doc1)]),
            Document(vec![Node::Dictionary(doc2)])
        ]));
    }

    #[test]
    fn test_parse_document_end_marker_with_comments() {
        let mut source = Buffer::new(b"# Comment before\nkey: value\n---\n# After doc\nother: 1");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert("key".to_string(), Node::Str("value".to_string()));
        let mut doc2 = HashMap::new();
        doc2.insert("other".to_string(), Node::Number(Numeric::Integer(1)));
        assert_eq!(result, Node::Documents(vec![
            Document(vec![
                Node::Comment("Comment before".to_string()),
                Node::Dictionary(doc1)
            ]),
            Document(vec![
                Node::Comment("After doc".to_string()),
                Node::Dictionary(doc2)
            ])
        ]));
    }

    #[test]
    fn test_parse_document_end_marker_only() {
        let mut source = Buffer::new(b"---");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_multiple_document_end_markers() {
        let mut source = Buffer::new(b"key: value\n---\n---\nother: 1");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert("key".to_string(), Node::Str("value".to_string()));
        let mut doc3 = HashMap::new();
        doc3.insert("other".to_string(), Node::Number(Numeric::Integer(1)));
        assert_eq!(result, Node::Documents(vec![
            Document(vec![Node::Dictionary(doc1)]),
            Document(vec![]),
            Document(vec![Node::Dictionary(doc3)])
        ]));
    }
    #[test]
    fn test_parse_nested_mapping_within_sequence() {
        let mut source = Buffer::new(
            b"people:\n  - name: John\n    likes:\n      - apples\n      - bananas\n"
        );
        let result = parse(&mut source).unwrap();

        let mut likes = Vec::new();
        likes.push(Node::Str("apples".to_string()));
        likes.push(Node::Str("bananas".to_string()));

        let mut john_map = HashMap::new();
        john_map.insert("name".to_string(), Node::Str("John".to_string()));
        john_map.insert("likes".to_string(), Node::Array(likes));

        let mut people_seq = Vec::new();
        people_seq.push(Node::Dictionary(john_map));

        let mut outer_map = HashMap::new();
        outer_map.insert("people".to_string(), Node::Array(people_seq));

        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Dictionary(outer_map)])])
        );


    }

    #[test]
    fn test_parse_sequence_of_mappings() {
        let yaml = b"-\n  name: Mark Joseph\n  hr: 87\n  avg: 0.278\n-\n  name: James Stephen\n  hr: 63\n  avg: 0.288\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut mark_map = HashMap::new();
        mark_map.insert("name".to_string(), Node::Str("Mark Joseph".to_string()));
        mark_map.insert("hr".to_string(), Node::Number(Numeric::Integer(87)));
        mark_map.insert("avg".to_string(), Node::Number(Numeric::Float(0.278)));

        let mut james_map = HashMap::new();
        james_map.insert("name".to_string(), Node::Str("James Stephen".to_string()));
        james_map.insert("hr".to_string(), Node::Number(Numeric::Integer(63)));
        james_map.insert("avg".to_string(), Node::Number(Numeric::Float(0.288)));

        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Dictionary(mark_map),
            Node::Dictionary(james_map),
        ])])]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_peek_ahead_for_mapping_key_basic() {
        let mut source = Buffer::new(b"key: value");
        assert_eq!(source.get_current_indent_level(), 0);
        assert!(peek_ahead_for_mapping_key(&mut source));
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_no_colon() {
        let mut source = Buffer::new(b"key value");
        assert!(!peek_ahead_for_mapping_key(&mut source));
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_colon_after_newline() {
        let mut source = Buffer::new(b"key\n: value");
        assert!(!peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_spaces_before_colon() {
        let mut source = Buffer::new(b"key   : value");
        assert!(peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_empty() {
        let mut source = Buffer::new(b"");
        assert!(!peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_parse_inline_mapping_top_level() {
        let mut source = Buffer::new(b"{a: 1, b: 2}");
        let result = parse(&mut source).unwrap();
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::Number(Numeric::Integer(1)));
        map.insert("b".to_string(), Node::Number(Numeric::Integer(2)));
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(map)])]));
    }

    #[test]
    fn test_parse_inline_mapping_empty() {
        let mut source = Buffer::new(b"{}");
        let result = parse(&mut source).unwrap();
        let map: HashMap<String, Node> = HashMap::new();
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(map)])]));
    }

    #[test]
    fn test_parse_inline_mapping_as_value() {
        let mut source = Buffer::new(b"parent: {a: 1, b: test}");
        let result = parse(&mut source).unwrap();
        let mut child = HashMap::new();
        child.insert("a".to_string(), Node::Number(Numeric::Integer(1)));
        child.insert("b".to_string(), Node::Str("test".to_string()));
        let mut parent = HashMap::new();
        parent.insert("parent".to_string(), Node::Dictionary(child));
        assert_eq!(result, Node::Documents(vec![Document(vec![Node::Dictionary(parent)])]));
    }
}


