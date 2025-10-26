use crate::io::traits::IDestination;
use crate::nodes::node::*;

// Escape string for double-quoted YAML scalars.
fn escape_double(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut iter = s.chars().peekable();
    while let Some(c) = iter.next() {
        match c {
            // Preserve literal newlines to support multi-line flow scalars
            '\n' => out.push('\n'),
            '\r' => {
                out.push('\\');
                out.push('r');
            }
            '\t' => {
                out.push('\\');
                out.push('t');
            }
            '\\' => {
                // If this is a known YAML escape (n, r, t, b, x), emit as-is
                if let Some(&next) = iter.peek() {
                    match next {
                        'n' | 'r' | 't' | 'b' | 'x' => {
                            out.push('\\');
                            out.push(next);
                            iter.next(); // consume the peeked char
                        }
                        _ => {
                            out.push('\\');
                            out.push('\\');
                        }
                    }
                } else {
                    out.push('\\');
                    out.push('\\');
                }
            }
            '"' => {
                out.push('\\');
                out.push('"');
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
        if c == '\'' {
            out.push('\'');
            out.push('\'');
        } else {
            out.push(c);
        }
    }
    out
}

// Normalize newlines by removing CR characters so inputs read from files
// with CRLF line endings don't emit stray '\r' characters when stringified.
fn normalize_newlines(s: &str) -> String {
    s.replace('\r', "")
}

fn stringify_document_with_indent(
    node: &Node,
    destination: &mut dyn IDestination,
    indent: usize,
) -> Result<(), String> {
    let indent_str = "  ".repeat(indent);
    match node {
        Node::None => destination.add_bytes(&format!("{}null", indent_str)),
        Node::Boolean(b) => destination.add_bytes(&format!("{}{}", indent_str, b)),
        Node::Str(s, qt, style) => {
            // Normalize CRLF -> LF by removing CR so file-based input using
            // Windows line endings doesn't leak '\r' into the output.
            let s = normalize_newlines(s);
            match qt {
                QuoteType::Double => {
                    // escape common sequences for double-quoted output
                    destination.add_bytes(&format!("{}\"{}\"", indent_str, escape_double(&s)))
                }
                QuoteType::Single => {
                    // Prefer double quotes only for single-line content that contains
                    // a single quote or backslash. Preserve single quotes for multiline
                    // scalars to match expected output semantics.
                    if !s.contains('\n') && (s.contains('\'') || s.contains('\\')) {
                        destination.add_bytes(&format!("{}\"{}\"", indent_str, escape_double(&s)))
                    } else {
                        // In single-quoted YAML scalars, single quotes are represented by doubling them
                        destination.add_bytes(&format!("{}'{}'", indent_str, escape_single(&s)))
                    }
                }
                QuoteType::Unquoted => {
                    // Emit literal block scalars '|' when content is multiline OR when style is explicitly Literal.
                    if s.contains('\n') || matches!(style, BlockStyle::Literal) {
                        let content_indent = "  ".repeat(indent + 1);
                        destination.add_bytes(&format!("{}|{}\n", indent_str, ""));

                        // The parser has already normalized indentation for block literals.
                        // Simply emit each line, prefixing content indent for non-empty lines
                        // and using a bare newline for empty lines.
                        let lines: Vec<&str> = s.split('\n').collect();
                        if !s.contains('\n') && matches!(style, BlockStyle::Literal) {
                            // Single-line literal: trim leading spaces only for display
                            let line = s.trim_start();
                            destination.add_bytes(&format!("{}{}\n", content_indent, line));
                        } else {
                            for line in lines {
                                if line.is_empty() {
                                    destination.add_bytes("\n");
                                } else {
                                    destination.add_bytes(&format!("{}{}\n", content_indent, line));
                                }
                            }
                        }
                    } else {
                        destination.add_bytes(&format!("{}{}", indent_str, s))
                    }
                }
            }
        }
        Node::Comment(c) => {
            // Normalize comments as well to avoid CR characters from file sources
            let c = normalize_newlines(c);
            destination.add_bytes(&format!("{}# {}", indent_str, c))
        }
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
                    Node::Str(_, QuoteType::Unquoted, BlockStyle::Literal) => {
                        // Literal block already emits its own trailing newline lines; don't add another
                        stringify_document_with_indent(value, destination, 0)?;
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
        Node::Anchored(inner, name) => {
            // Emit an anchor before the node
            destination.add_bytes(&format!("&{} ", name));
            stringify_document_with_indent(inner, destination, indent)?;
        }
        Node::Alias(name) => {
            destination.add_bytes(&format!("*{}", name));
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
        Node::Array(items) => items.iter().all(|n| node_is_blank(n)),
        Node::Mapping(pairs) => pairs.is_empty(),
        Node::Document(nodes) => nodes.iter().all(|n| node_is_blank(n)),
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
                    if nodes.iter().all(|n| node_is_blank(n)) {
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
                            destination.add_bytes("--- |\n");
                            for line in s.split('\n') {
                                destination.add_bytes(&format!("{}\n", line));
                            }
                            destination.add_bytes("...\n");
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
    use crate::{BufferSource, parse};
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestFile {
        path: String,
    }

    impl TestFile {
        fn new_with_content(content: &[u8]) -> Self {
            let id = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = format!("test_stringify_temp_{}.yaml", id);
            fs::write(&path, content).unwrap();
            Self { path }
        }

        fn new_empty() -> Self {
            let id = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = format!("test_stringify_temp_{}.yaml", id);
            fs::write(&path, b"").unwrap();
            Self { path }
        }

        fn path(&self) -> &str {
            &self.path
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

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
        stringify(
            &Node::Str("test".to_string(), QuoteType::Double, BlockStyle::None),
            &mut dest,
        )
        .unwrap();
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
            Node::Str("test".to_string(), QuoteType::Double, BlockStyle::None),
        ];
        stringify(&Node::Array(arr), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "- 1\n- \"test\"\n");
    }

    #[test]
    fn test_stringify_mapping() {
        let mut dest = Buffer::new();
        let mapping = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Double, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Double, BlockStyle::None),
        )]);
        stringify(&mapping, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"key\": \"value\"\n");
    }

    #[test]
    fn test_stringify_documents() {
        let mut dest = Buffer::new();
        let docs = vec![
            Node::Str("doc1".to_string(), QuoteType::Double, BlockStyle::None),
            Node::Str("doc2".to_string(), QuoteType::Double, BlockStyle::None),
        ];
        stringify(&Node::Documents(docs), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n\"doc1\"...\n---\n\"doc2\"...\n");
    }

    #[test]
    fn test_stringify_anchor_and_alias() {
        let mut dest = Buffer::new();
        // Build an anchored mapping value and an alias node
        let anchored = Node::Anchored(
            Box::new(Node::Mapping(vec![(
                Node::Str("nested".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])),
            "a".to_string(),
        );

        let docs = vec![anchored, Node::Alias("a".to_string())];
        stringify(&Node::Documents(docs), &mut dest).unwrap();
        // Expect anchors and aliases to be emitted
        let out = dest.to_string();
        assert!(out.contains("&a"));
        assert!(out.contains("*a"));
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

    #[test]
    fn test_stringify_anchor_alias_hr_rbi() {
        use crate::io::sources::buffer::Buffer as SrcBuffer;
        // YAML from testfile016.yaml
        let yaml = b"---\nhr:\n  - Mark McGwire\n  # Following node labeled SS\n  - &SS Sammy Sosa\nrbi:\n  - *SS # Subsequent occurance\n  - Ken Griffey\n";
        let mut source = SrcBuffer::new(yaml);
        let node = crate::parse(&mut source).unwrap();

        let mut dest = crate::io::destinations::buffer::Buffer::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        let expected = "---\nhr: \n  - Mark McGwire\n  - Sammy Sosa\nrbi: \n  - Sammy Sosa\n  - Ken Griffey\n...\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_stringify_anchor_alias_hr_rbi_from_file() {
        use crate::io::sources::file::File as FileSource;

        // Create a temporary file with the YAML contents (same as testfile016.yaml)
        let input = b"---\r\nhr:\r\n  - Mark McGwire\r\n  # Following node labeled SS\r\n  - &SS Sammy Sosa\r\nrbi:\r\n  - *SS # Subsequent occurance\r\n  - Ken Griffey\r\n";
        let in_file = TestFile::new_with_content(input);

        // parse from file source
        let mut source = FileSource::new(in_file.path()).unwrap();
        let node = parse(&mut source).unwrap();

        // stringify to an in-memory buffer and compare
        let mut dest = crate::io::destinations::buffer::Buffer::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        let expected = "---\nhr: \n  - Mark McGwire\n  - Sammy Sosa\nrbi: \n  - Sammy Sosa\n  - Ken Griffey\n...\n";
        assert_eq!(out, expected);
    }
    #[test]
    fn test_with_comment_header() {
        let mut dest = Buffer::new();
        let mut source = BufferSource::new(
            "# Ranking of 1998 home runs\n---\n- Mark Joseph\n- James Stephen\n- Ken Griffey\n"
                .as_bytes(),
        );
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\n- Mark Joseph\n- James Stephen\n- Ken Griffey\n...\n"
        );
    }

    #[test]
    fn test_stringify_double_quoted_multiline_scalar() {
        let mut dest = Buffer::new();
        let node = Node::Str(
            "line1\nline2".to_string(),
            QuoteType::Double,
            BlockStyle::None,
        );
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"line1\nline2\"");
    }

    #[test]
    fn test_stringify_single_quoted_multiline_and_escaping() {
        let mut dest = Buffer::new();
        let node = Node::Str(
            "O'Reilly\nBooks".to_string(),
            QuoteType::Single,
            BlockStyle::None,
        );
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "'O''Reilly\nBooks'");
    }

    #[test]
    fn test_stringify_mapping_with_multiline_value() {
        let mut dest = Buffer::new();
        let mapping = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("a\nb".to_string(), QuoteType::Double, BlockStyle::None),
        )]);
        stringify(&mapping, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "key: \"a\nb\"\n");
    }

    #[test]
    fn test_stringify_sequence_with_multiline_item() {
        let mut dest = Buffer::new();
        let seq = Node::Array(vec![Node::Str(
            "a\nb".to_string(),
            QuoteType::Double,
            BlockStyle::None,
        )]);
        stringify(&seq, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "- \"a\nb\"\n");
    }

    #[test]
    fn test_stringify_mapping_with_inline_comment_and_indented_sequence_files() {
        // Mirror parser test: ensure stringifier emits the mapping -> sequence correctly using files
        use crate::io::destinations::file::File as FileDestination;
        use crate::io::sources::file::File as FileSource;

        let input = b"---\r\nhr: # 1998 hr ranking\r\n  - Mark McGwire\r\n  - Sammy Sosa\n";
        let in_file = TestFile::new_with_content(input);
        let out_file = TestFile::new_empty();

        // parse from file source
        let mut source = FileSource::new(in_file.path()).unwrap();
        let node = parse(&mut source).unwrap();

        // stringify to file destination
        let mut dest = FileDestination::new(out_file.path()).unwrap();
        stringify(&node, &mut dest).unwrap();

        // read output and compare
        let out = fs::read_to_string(out_file.path()).unwrap();
        let expected = "---\nhr: \n  - Mark McGwire\n  - Sammy Sosa\n...\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_stringify_explicit_sequence_keys() {
        // Hardcoded YAML input (from files/testfile017.yaml)
        let yaml = b"? # PLAY SCHEDULE\n  - Detroit Tigers\n  - Chicago Cubs\n:\n  - 2001-07-23\n\n? [ New York Yankees,\n    Atlanta Braves ]\n: [ 2001-07-02, 2001-08-12,\n    2001-08-14 ]\n";
        let mut src = BufferSource::new(yaml.as_ref());
        let node = parse(&mut src).expect("parse");

        // stringify to buffer using the parser-produced node
        let mut dest = crate::io::destinations::buffer::Buffer::new();
        stringify(&node, &mut dest).expect("");
        let out = normalize_newlines(&dest.to_string());

        // Hardcoded expected stringify output (from files/testfile017.yaml.stringify)
        let expected = normalize_newlines(
            "---\n\"[Detroit Tigers, Chicago Cubs]\": \n  - 2001-07-23\n\"[New York Yankees, Atlanta Braves]\": \n  - 2001-07-02\n  - 2001-08-12\n  - 2001-08-14\n...\n",
        );

        assert_eq!(out, expected);
    }

    // Tests moved from parser::default that asserted stringify output of parse results
    #[test]
    fn test_stringify_from_parser_literal_block_literal_scalar_with_indent() {
        use crate::io::destinations::buffer::Buffer as DestBuffer;
        use crate::io::sources::buffer::Buffer as SrcBuffer;

        let yaml = b"---\nstring1: |\n  Line1\n  line2\n";
        let mut source = SrcBuffer::new(yaml);
        let node = crate::parse(&mut source).unwrap();

        let mut dest = DestBuffer::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert_eq!(out, "---\nstring1: |\n  Line1\n  line2\n...\n");
    }

    #[test]
    fn test_stringify_from_parser_literal_block_folded_scalar_with_indent() {
        use crate::io::destinations::buffer::Buffer as DestBuffer;
        use crate::io::sources::buffer::Buffer as SrcBuffer;

        let yaml = b"---\nstring1: >\n  Line1\n  line2\n";
        let mut source = SrcBuffer::new(yaml);
        let node = crate::parse(&mut source).unwrap();

        let mut dest = DestBuffer::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        // parser folded '>' to a literal block in stringify behavior
        assert_eq!(out, "---\nstring1: |\n  Line1 line2\n...\n");
    }

    #[test]
    fn test_stringify_merge_key_with_single_alias() {
        use crate::io::destinations::buffer::Buffer as DestBuffer;
        use crate::io::sources::buffer::Buffer as SrcBuffer;

        let yaml = b"---\na: &a\n  nested: anchor\nparent:\n  <<: *a\n  key: value\n";
        let mut source = SrcBuffer::new(yaml);
        let node = crate::parse(&mut source).unwrap();

        let mut dest = DestBuffer::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert!(
            out.contains("<<:"),
            "stringified output should contain merge key: {}",
            out
        );
    }
}
