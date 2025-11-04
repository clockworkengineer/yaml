//! Document-level YAML parser split into several focused source files.
//! This module re-exports the primary parsing entry points used by the rest
//! of the crate and contains unit tests ported from the former single-file
//! implementation.

mod anchors;
mod helpers;
mod inline;
mod mapping;
mod scalar;
mod sequence;
mod value;

pub(crate) use anchors::{collect_anchors, expand_merge_keys, replace_aliases};
#[cfg(test)]
pub(crate) use helpers::parse_quoted_scalar;
pub(crate) use helpers::{parse_comment, peek_ahead_for_mapping_key};
pub(crate) use inline::{parse_inline_mapping, parse_inline_sequence};
pub(crate) use mapping::parse_mapping;
#[cfg(test)]
pub(crate) use scalar::parse_scalar;
pub(crate) use sequence::parse_sequence;
pub(crate) use value::parse_value;

use crate::constants::*;
use crate::io::traits::ISource;
use crate::nodes::node::BlockStyle;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use std::collections::HashMap;

use helpers::node_is_blank;
use helpers::skip_whitespace;

/// Parses the contents of a YAML document based on the current character and context.
///
/// Determines the appropriate parsing strategy based on the current character:
/// sequences (-), comments (#), inline mappings ({}), inline sequences ([]),
/// explicit mapping keys (?), block scalars (| or >), or regular mappings.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The current indentation level for proper nesting
///
/// # Returns
///
/// Result containing the parsed Node or an error string
pub fn parse_document_contents(
    source: &mut dyn ISource,
    indent_level: usize,
) -> Result<Node, String> {
    match source.current() {
        Some(c) if c == '-' => {
            let indent_level = source.get_current_indent_level();
            Ok(parse_sequence(source, indent_level)?)
        }
        Some(c) if c == '#' => {
            parse_comment(source);
            skip_whitespace(source);
            parse_document_contents(source, indent_level)
        }
        Some(c) if c == '{' => Ok(parse_inline_mapping(source)?),
        Some(c) if c == '[' => Ok(parse_inline_sequence(source)?),
        Some(c) if c == '?' => {
            source.next();
            skip_whitespace(source);
            let mut key_node: Node;

            if source.current() == Some('[') {
                key_node = parse_inline_sequence(source)?;
            } else if source.current() == Some('-') {
                let nested_indent = source.get_current_indent_level();
                key_node = parse_sequence(source, nested_indent)?;
            } else if matches!(source.current(), Some('|') | Some('>')) {
                let is_folded = source.current() == Some('>');

                let _ = crate::utils::collect_until(source, |c| c == '\n');
                if source.current() == Some('\n') {
                    source.next();
                }

                let mut raw_lines: Vec<String> = Vec::new();
                let mut first_indent: Option<usize> = None;
                loop {
                    if source.current().is_none() {
                        break;
                    }
                    let st_line = source.save_state();
                    let mut cur_indent = 0usize;
                    while let Some(CHAR_SPACE) = source.current() {
                        cur_indent += 1;
                        source.next();
                    }
                    let cur_is_newline = source.current() == Some('\n');

                    let is_colon_start = matches!(source.current(), Some(':'));
                    source.restore_state(st_line);

                    if first_indent.is_none() {
                        if cur_is_newline {
                            let _ = crate::utils::collect_until(source, |c| c == '\n');
                            if source.current() == Some('\n') {
                                source.next();
                            }
                            raw_lines.push(String::new());
                            continue;
                        } else {
                            first_indent = Some(cur_indent);
                        }
                    } else if !cur_is_newline && cur_indent < first_indent.unwrap() {
                        break;
                    }

                    if is_colon_start {
                        break;
                    }

                    let raw_line = crate::utils::collect_until(source, |c| c == '\n');
                    if source.current() == Some('\n') {
                        source.next();
                    }
                    raw_lines.push(raw_line);
                }

                while matches!(raw_lines.last(), Some(s) if s.is_empty()) {
                    raw_lines.pop();
                }

                let fi = first_indent.unwrap_or(0);
                let mut norm_lines: Vec<String> = Vec::with_capacity(raw_lines.len());
                if is_folded {
                    for l in raw_lines.iter() {
                        if l.is_empty() {
                            norm_lines.push(String::new());
                        } else {
                            let lead = l.chars().take_while(|&ch| ch == CHAR_SPACE).count();
                            let strip = fi.min(lead);
                            let stripped: String = l.chars().skip(strip).collect();
                            norm_lines.push(stripped);
                        }
                    }
                } else {
                    norm_lines = raw_lines.clone();
                }

                let mut escaped_parts: Vec<String> = Vec::with_capacity(norm_lines.len());
                for l in norm_lines.iter() {
                    let stripped: String = l.chars().skip(fi).collect();
                    escaped_parts.push(stripped);
                }

                let mut escaped_key = escaped_parts.join("\\n");
                escaped_key.push_str("\\n");
                key_node = Node::Str(
                    escaped_key,
                    crate::nodes::node::QuoteType::Double,
                    crate::nodes::node::BlockStyle::None,
                );
            } else if matches!(source.current(), Some('"') | Some('\'')) {
                key_node = crate::parser::document::value::parse_value(source)?;
            } else if source.current() == Some('#') || source.current() == Some('\n') {
                let st = source.save_state();
                let _ = crate::utils::read_line_trimmed_into_string(source);
                if source.current() == Some('\n') {
                    source.next();
                }
                skip_whitespace(source);
                if source.current() == Some('-') {
                    let nested_indent = source.get_current_indent_level();
                    key_node = parse_sequence(source, nested_indent)?;
                } else {
                    source.restore_state(st);
                    key_node = Node::Str(
                        crate::utils::read_line_trimmed_into_string(source),
                        crate::nodes::node::QuoteType::Unquoted,
                        crate::nodes::node::BlockStyle::None,
                    );
                }
            } else if source.current().is_some() {
                key_node = Node::Str(
                    crate::utils::read_line_trimmed_into_string(source),
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::None,
                );
            } else {
                key_node = Node::Str(
                    String::new(),
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::None,
                );
            }

            match key_node {
                Node::Array(_) | Node::Mapping(_) => {
                    let inline = helpers::node_to_inline_string(&key_node);
                    key_node = Node::Str(
                        inline,
                        crate::nodes::node::QuoteType::Double,
                        crate::nodes::node::BlockStyle::None,
                    );
                }
                Node::Str(s, _qt, style) => {
                    let key_string = if matches!(style, BlockStyle::Literal) {
                        format!("{}\n", s)
                    } else {
                        s
                    };
                    key_node = Node::Str(
                        key_string,
                        crate::nodes::node::QuoteType::Double,
                        crate::nodes::node::BlockStyle::None,
                    );
                }
                other => {
                    let inline = helpers::node_to_inline_string(&other);
                    key_node = Node::Str(
                        inline,
                        crate::nodes::node::QuoteType::Double,
                        crate::nodes::node::BlockStyle::None,
                    );
                }
            }

            let st_colon = source.save_state();
            let mut found_colon = false;
            loop {
                skip_whitespace(source);
                match source.current() {
                    Some(':') => {
                        source.next();
                        found_colon = true;
                        break;
                    }
                    Some('#') => {
                        parse_comment(source);
                        if source.current() == Some('\n') {
                            source.next();
                        }
                        continue;
                    }
                    Some('\n') => {
                        source.next();
                        continue;
                    }
                    Some(_) | None => break,
                }
            }
            if !found_colon {
                source.restore_state(st_colon);
                if source.current() == Some('\n') {
                    source.next();
                }
                loop {
                    skip_whitespace(source);
                    if source.current() == Some(':') {
                        break;
                    }
                    if source.current().is_none() {
                        break;
                    }
                    crate::utils::skip_until_newline(source);
                    if source.current().is_none() {
                        break;
                    }
                }
            }
            if source.current() == Some(':') {
                source.next();
            }
            skip_whitespace(source);
            let mut value_node = match source.current() {
                Some('[') => parse_inline_sequence(source)?,
                Some('{') => parse_inline_mapping(source)?,
                Some('-') => {
                    let nested_indent = source.get_current_indent_level();
                    parse_sequence(source, nested_indent)?
                }
                Some(_) => parse_value(source)?,
                None => {
                    return Err(helpers::parse_error(
                        source,
                        "Unexpected end of input while parsing explicit pair value",
                    ));
                }
            };

            if matches!(value_node, Node::None) {
                let st_peek = source.save_state();
                crate::utils::skip_whitespace_and_comments(source);
                if source.current() == Some('-') {
                    let nested_indent = source.get_current_indent_level();
                    value_node = parse_sequence(source, nested_indent)?;
                } else {
                    source.restore_state(st_peek);
                }
            }

            let mut pairs: Vec<(Node, Node)> = Vec::new();
            pairs.push((key_node, value_node));
            Ok(Node::Mapping(pairs))
        }
        Some(c) if c.is_alphanumeric() => {
            if peek_ahead_for_mapping_key(source) {
                Ok(parse_mapping(source, indent_level)?)
            } else if indent_level > 0 {
                let base_indent = source.get_current_indent_level();
                let mut parts: Vec<String> = Vec::new();
                loop {
                    let line = crate::utils::read_line_trimmed_into_string(source);
                    if !line.is_empty() {
                        parts.push(line);
                    }
                    if source.current() == Some('\n') {
                        source.next();
                    }
                    let st = source.save_state();
                    skip_whitespace(source);
                    let cur_indent = source.get_current_indent_level();
                    let next_char = source.current();
                    source.restore_state(st);
                    if next_char.is_none() || cur_indent < base_indent {
                        break;
                    }
                    if matches!(
                        next_char,
                        Some('-') | Some('{') | Some('[') | Some('?') | Some('#')
                    ) {
                        break;
                    }
                }
                let joined = parts.join(" ");
                Ok(Node::Str(
                    joined,
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::None,
                ))
            } else {
                Ok(parse_mapping(source, indent_level)?)
            }
        }
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(source, indent_level)?)
        }
        Some('\0') => {
            source.next();
            Ok(parse_document_contents(source, indent_level)?)
        }
        Some(c) if matches!(c, '<' | '>' | '"' | '\'' | '|') => {
            if matches!(source.current(), Some('"') | Some('\''))
                && peek_ahead_for_mapping_key(source)
            {
                Ok(parse_mapping(source, indent_level)?)
            } else {
                Ok(parse_value(source)?)
            }
        }
        Some(c) => Err(helpers::parse_error(
            source,
            &format!(
                "{}{}",
                crate::error::messages::ERR_UNEXPECTED_CHAR_PREFIX,
                c
            ),
        )),
        None => Ok(Node::None),
    }
}

/// Parses a single YAML document from the source.
///
/// Processes document content while handling document start/end markers (--- and ...),
/// comments, and various node types. Collects all document nodes and performs
/// post-processing including anchor resolution and merge key expansion.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The indentation level for the document
///
/// # Returns
///
/// Result containing a Document Node or an error string
pub fn parse_document(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    skip_whitespace(source);

    let mut document_nodes = Vec::new();

    while let Some(c) = source.current() {
        if (c == '-' || c == '.')
            && crate::parser::document::helpers::peek_ahead_for_document_start_end(source, c)
        {
            crate::utils::skip_until_newline(source);
            skip_whitespace(source);
            break;
        }

        match c {
            '#' => {
                parse_comment(source);
                skip_whitespace(source);
                continue;
            }
            _ => {
                let node = parse_document_contents(source, indent_level)?;
                if !node_is_blank(&node) {
                    document_nodes.push(node);
                }
            }
        }
    }

    let mut normalized_nodes: Vec<Node> = Vec::new();
    let mut i = 0usize;
    while i < document_nodes.len() {
        if i + 1 < document_nodes.len() {
            if let Node::Mapping(pairs) = &document_nodes[i] {
                if pairs.len() == 1 && matches!(pairs[0].1, Node::None) {
                    if let Node::Array(arr) = &document_nodes[i + 1] {
                        let key = pairs[0].0.clone();
                        normalized_nodes.push(Node::Mapping(vec![(key, Node::Array(arr.clone()))]));
                        i += 2;
                        continue;
                    }
                }
            }
        }
        normalized_nodes.push(document_nodes[i].clone());
        i += 1;
    }

    let mut doc_node = Document(normalized_nodes);

    let mut anchors: HashMap<String, Node> = HashMap::new();
    collect_anchors(&doc_node, &mut anchors)?;

    expand_merge_keys(&mut doc_node, &anchors)?;
    replace_aliases(&mut doc_node, &anchors)?;

    Ok(doc_node)
}

/// Main entry point for parsing YAML content from a source.
///
/// Parses one or more YAML documents from the source, handling document
/// separators and creating a Documents node containing all parsed documents.
/// Empty or blank documents are filtered out automatically.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// Result containing a Documents Node with all parsed documents or an error string
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    let mut docs: Vec<Node> = Vec::new();
    if helpers::peek_ahead_for_document_start_end(source, '-') {
        source.next();
        source.next();
        source.next();
        if source.current() == Some(' ') {
            source.next();
        }
    }
    while source.more() {
        let document = parse_document(source, 0);
        match document {
            Ok(doc) => {
                let is_blank_doc = match &doc {
                    Document(nodes) => nodes.iter().all(node_is_blank),
                    _ => false,
                };
                if !is_blank_doc {
                    docs.push(doc)
                }
            }
            Err(err) => return Err(err),
        }
    }
    if docs.is_empty() {
        docs.push(Document(Vec::new()))
    }
    Ok(Node::Documents(docs))
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
        assert_eq!(
            parse_scalar("42"),
            Node::Number(crate::nodes::node::Numeric::Integer(42))
        );
        assert_eq!(
            parse_scalar("3.14"),
            Node::Number(crate::nodes::node::Numeric::Float(3.14))
        );
        assert_eq!(
            parse_scalar("hello"),
            Node::Str(
                "hello".to_string(),
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None
            )
        );
        assert_eq!(
            parse_scalar("#comment"),
            Node::Str(
                "#comment".to_string(),
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None
            )
        );
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
    fn test_parse_quoted_scalar_single_and_double() {
        let mut s1 = Buffer::new(b"'single''quote'");
        let r1 = parse_quoted_scalar(&mut s1).unwrap();
        assert_eq!(r1, "'single''quote'");

        let mut s2 = Buffer::new(b"\"double\\\"quote\"");
        let r2 = parse_quoted_scalar(&mut s2).unwrap();
        assert_eq!(r2, "\"double\\\"quote\"");
    }

    #[test]
    fn test_parse_inline_sequence_simple_and_empty() {
        let mut src = Buffer::new(b"[1, 'two', 3]");
        let node = parse_inline_sequence(&mut src).unwrap();
        assert!(matches!(node, Node::Array(_)));
        if let Node::Array(items) = node {
            assert_eq!(items.len(), 3);
            assert!(matches!(
                items[0],
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
            assert!(matches!(
                items[1],
                Node::Str(_, crate::nodes::node::QuoteType::Single, _)
            ));
            assert!(matches!(
                items[2],
                Node::Number(crate::nodes::node::Numeric::Integer(3))
            ));
        }

        let mut empty = Buffer::new(b"[]");
        let node = parse_inline_sequence(&mut empty).unwrap();
        assert!(matches!(node, Node::Array(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_inline_mapping_simple_and_empty() {
        let mut src = Buffer::new(b"{key1: 1, 'key2': \"two\"}");
        let node = parse_inline_mapping(&mut src).unwrap();
        assert!(matches!(node, Node::Mapping(_)));
        if let Node::Mapping(pairs) = node {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(
                pairs[0].0,
                Node::Str(_, crate::nodes::node::QuoteType::Unquoted, _)
            ));
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
            assert!(matches!(
                pairs[1].0,
                Node::Str(_, crate::nodes::node::QuoteType::Single, _)
            ));
            assert!(matches!(pairs[1].1, Node::Str(_, _, _)));
        }

        let mut empty = Buffer::new(b"{}");
        let node = parse_inline_mapping(&mut empty).unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_comment_trims_hash_and_newline() {
        let mut src = Buffer::new(b"# Hello world  \n");
        let text = parse_comment(&mut src);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_parse_value_alias_and_anchor() {
        let mut a = Buffer::new(b"*myalias");
        let n = parse_value(&mut a).unwrap();
        assert!(matches!(n, Node::Alias(ref name) if name == "myalias"));

        let mut b = Buffer::new(b"&aname 42");
        let n = parse_value(&mut b).unwrap();
        if let Node::Anchored(inner, name) = n {
            assert_eq!(*name, "aname".to_string());
            assert!(matches!(
                *inner,
                Node::Number(crate::nodes::node::Numeric::Integer(42))
            ));
        } else {
            panic!("expected Anchored node");
        }
    }

    #[test]
    fn test_parse_document_contents_empty_line() {
        let mut src = Buffer::new(b"key: value\n\n");
        let n = parse_document_contents(&mut src, 0).unwrap();
        assert!(matches!(n, Node::Mapping(_)));
    }
}
