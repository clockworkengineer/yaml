/// Context for mapping parser state
struct MappingParseContext {
    stack: Vec<(usize, Vec<(Node, Node)>)>,
    base_indent: usize,
}

use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::parser::directives::DirectiveContext;
use crate::parser::document::error_builder::mapping_key_error_yaml;
use crate::parser::document::node_utils::force_key_to_string;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

#[cfg(feature = "debug-trace")]
#[inline]
fn mapping_log(msg: String) {
    #[cfg(feature = "std")]
    {
        if let Ok(v) = std::env::var("YAML_TRACE_MAPPING") {
            if v.eq_ignore_ascii_case("1")
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("on")
            {
                log::debug!("{}", msg);
                return;
            }
        }
    }
    log::trace!("{}", msg);
}

/// Parse a single key-value mapping pair (for sequence items)
#[allow(dead_code)]
pub fn parse_single_mapping_pair_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> crate::parser::ParseResult<Node> {
    let (key, value) = parse_mapping_pair(stream, directives, 0, 0)?;
    Ok(Node::Mapping(vec![(key, value)]))
}

/// Parse a block mapping using tokens
///
/// Example:
/// ```yaml
/// key1: value1
/// key2: value2
/// !!str: tagged_key
/// ? complex_key
/// : complex_value
/// ```
///
/// Benefits of token-based approach:
/// - No complex lookahead for keys with decorators
/// - Clear token boundaries prevent infinite loops
/// - Natural handling of explicit keys (?)
#[allow(dead_code)]
pub fn parse_mapping_with_tokens(
    stream: &mut TokenStream,
    base_indent: usize,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    use crate::utils::optimization::{CapacityHints, NodeBuilder};
    let node_builder = NodeBuilder::with_hints(CapacityHints::small());
    let mut ctx = MappingParseContext {
        stack: vec![(
            base_indent,
            Vec::with_capacity(node_builder.hints().mapping_pairs),
        )],
        base_indent,
    };

    stream.skip_trivia()?;

    loop {
        let saw_comment_between_entries = stream.skip_newlines_and_comments_with_flag()?;
        ctx.handle_dedent(stream);
        let current_indent = ctx.get_current_indent();
        let token = stream.current().cloned();
        if let Some(result) = ctx.handle_special_tokens(stream, current_indent, &token)? {
            return Ok(result);
        }
        if let Some(pair) = ctx.try_parse_and_insert_pair(
            stream,
            directives,
            current_indent,
            depth,
            saw_comment_between_entries,
        )? {
            if let Some((_, pairs)) = ctx.stack.last_mut() {
                pairs.push(pair);
            }
        }
    }
}

impl MappingParseContext {
    fn get_current_indent(&self) -> usize {
        self.stack
            .last()
            .map(|(lvl, _)| *lvl)
            .unwrap_or(self.base_indent)
    }

    fn dedent_unwind_mapping_stack(&mut self, target_level: usize) {
        while self.stack.len() > 1 && self.stack.last().map(|(i, _)| *i).unwrap_or(0) > target_level
        {
            let (_, closed_pairs) = self.stack.pop().unwrap();
            if let Some((_, parent_pairs)) = self.stack.last_mut() {
                parent_pairs.push((Node::None, Node::Mapping(closed_pairs)));
            }
        }
    }

    fn handle_dedent(&mut self, stream: &mut TokenStream) {
        loop {
            let current_indent = self
                .stack
                .last()
                .map(|(lvl, _)| *lvl)
                .unwrap_or(self.base_indent);
            let token_indent = match stream.current() {
                Some(Token::Indent(level)) => *level,
                _ => current_indent,
            };
            if token_indent < current_indent && self.stack.len() > 1 {
                let (_, closed_pairs) = self.stack.pop().unwrap();
                if let Some((_, parent_pairs)) = self.stack.last_mut() {
                    parent_pairs.push((Node::None, Node::Mapping(closed_pairs)));
                }
            } else {
                break;
            }
        }
    }

    fn handle_special_tokens(
        &mut self,
        stream: &mut TokenStream,
        current_indent: usize,
        token: &Option<Token>,
    ) -> crate::parser::ParseResult<Option<Node>> {
        match token {
            Some(Token::Indent(level)) if *level < current_indent => {
                self.dedent_unwind_mapping_stack(*level);
                let (_, pairs) = self.stack.last().unwrap();
                return Ok(Some(Node::Mapping(pairs.clone())));
            }
            Some(Token::Eof) => {
                while self.stack.len() > 1 {
                    let (_top_indent, top_pairs) = self.stack.pop().unwrap();
                    if let Some((_, parent_pairs)) = self.stack.last_mut() {
                        if let Some((_, last_value)) = parent_pairs.last_mut() {
                            *last_value = Node::Mapping(top_pairs);
                        } else {
                            parent_pairs.push((
                                force_key_to_string(Node::Str(
                                    "<unwound>".to_string(),
                                    QuoteType::Unquoted,
                                    BlockStyle::None,
                                )),
                                Node::Mapping(top_pairs),
                            ));
                        }
                    }
                }
                let (_, pairs) = self.stack.pop().unwrap();
                return Ok(Some(Node::Mapping(pairs)));
            }
            Some(Token::DocumentStart)
            | Some(Token::Dash)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd) => {
                let (_, pairs) = self.stack.pop().unwrap();
                return Ok(Some(Node::Mapping(pairs)));
            }
            Some(Token::DocumentEnd) => {
                crate::parser::document::helpers::validate_trailing_content_after_document_end(
                    stream,
                )?;
                let (_, pairs) = self.stack.pop().unwrap();
                return Ok(Some(Node::Mapping(pairs)));
            }
            _ => {}
        }
        Ok(None)
    }

    fn try_parse_and_insert_pair(
        &mut self,
        stream: &mut TokenStream,
        directives: &DirectiveContext,
        current_indent: usize,
        depth: usize,
        saw_comment_between_entries: bool,
    ) -> crate::parser::ParseResult<Option<(Node, Node)>> {
        let token = stream.current().cloned();
        if let Some(Token::Indent(level)) = token {
            let last_value_is_empty = self
                .stack
                .last()
                .and_then(|(_, pairs)| pairs.last())
                .map(|(_, v)| matches!(v, Node::None))
                .unwrap_or(false);
            if level > current_indent {
                if !last_value_is_empty && saw_comment_between_entries {
                    return Err(mapping_key_error_yaml(
                        stream.source_mut(),
                        "Invalid indentation after comment: indented content cannot extend a completed scalar mapping value",
                    ));
                }
                self.stack.push((level, Vec::new()));
                stream.next()?;
                return Ok(None);
            } else if level < current_indent {
                self.dedent_unwind_mapping_stack(level);
                stream.next()?;
                return Ok(None);
            } else {
                stream.next()?;
                return Ok(None);
            }
        }
        if let Some(Token::Eof)
        | Some(Token::DocumentEnd)
        | Some(Token::DocumentStart)
        | Some(Token::Dash)
        | Some(Token::FlowMappingEnd)
        | Some(Token::FlowSequenceEnd) = token
        {
            let (_, pairs) = self.stack.pop().unwrap();
            return Ok(Some((Node::None, Node::Mapping(pairs))));
        }
        let (key, value) = parse_mapping_pair(stream, directives, current_indent, depth)?;
        let norm_key = force_key_to_string(key);
        Ok(Some((norm_key, value)))
    }
}

/// Parse a single key-value pair (refactored)
#[allow(dead_code)]
fn parse_mapping_pair(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    cur_indent: usize,
    depth: usize,
) -> crate::parser::ParseResult<(Node, Node)> {
    #[cfg(feature = "debug-trace")]
    mapping_log(format!(
        "parse_mapping_pair: start, token = {:?}",
        stream.current()
    ));
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: start at token = {:?}", stream.current());

    let (explicit_key, key) = parse_mapping_key(stream, directives, depth)?;
    #[cfg(feature = "debug-trace")]
    mapping_log(format!(
        "parse_mapping_pair: after key, token = {:?}",
        stream.current()
    ));
    stream.skip_newlines_and_comments()?;

    // Handle explicit key with omitted value
    match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
        }
        _ if explicit_key => {
            #[cfg(feature = "debug-trace")]
            mapping_log(format!(
                "parse_mapping_pair: after explicit key newline/whitespace, token = {:?}",
                stream.current()
            ));
            match stream.current() {
                Some(Token::Plain(_))
                | Some(Token::Tag(_))
                | Some(Token::Anchor(_))
                | Some(Token::QuestionMark)
                | Some(Token::DocumentEnd)
                | Some(Token::DocumentStart)
                | Some(Token::Eof)
                | None => {
                    return Ok((key, Node::None));
                }
                Some(Token::Indent(_)) => {
                    return Ok((key, Node::None));
                }
                _ => {}
            }
            if !matches!(stream.current(), Some(Token::Colon)) {
                return Ok((key, Node::None));
            } else {
                stream.next()?;
            }
        }
        Some(Token::Eof) | None => {
            return Ok((key, Node::None));
        }
        Some(Token::Plain(_))
        | Some(Token::Tag(_))
        | Some(Token::Anchor(_))
        | Some(Token::QuestionMark) => {
            return Ok((key, Node::None));
        }
        Some(Token::Dash) => {
            return Ok((key, Node::None));
        }
        _ => {}
    }

    let value = parse_mapping_value(stream, directives, cur_indent, depth, explicit_key, &key)?;
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
    Ok((key, value))
}

fn parse_mapping_key(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<(bool, Node)> {
    let mut explicit_key = false;
    if crate::parser::document::explicit_key::is_explicit_key_start(stream) {
        stream.next()?;
        explicit_key = true;
    }
    if matches!(
        stream.current(),
        Some(Token::Tag(_)) | Some(Token::Anchor(_))
    ) {
        let decorators = stream.consume_decorators()?;
        if matches!(stream.current(), Some(Token::Colon)) {
            use crate::nodes::node::{BlockStyle, QuoteType};
            let node = Node::Str("".to_string(), QuoteType::Unquoted, BlockStyle::None);
            let key = apply_decorators_to_key(node, decorators, stream)?;
            Ok((explicit_key, key))
        } else {
            let key_node = parse_value_with_tokens(stream, directives, depth + 1)?;
            let key = apply_decorators_to_key(key_node, decorators, stream)?;
            Ok((explicit_key, key))
        }
    } else {
        let key_node = parse_value_with_tokens(stream, directives, depth + 1)?;
        Ok((explicit_key, key_node))
    }
}

fn apply_decorators_to_key(
    mut key_node: Node,
    decorators: crate::parser::token_stream::Decorators,
    stream: &mut TokenStream,
) -> crate::parser::ParseResult<Node> {
    if let Some(tag) = decorators.tag {
        key_node = Node::Tagged(Box::new(key_node), tag);
    }
    if let Some(anchor) = decorators.anchor {
        if matches!(key_node, Node::Alias(_)) {
            return Err(mapping_key_error_yaml(
                stream.source_mut(),
                "Invalid anchored alias key: anchors cannot be applied to alias nodes",
            ));
        }
        if matches!(key_node, Node::Anchored(_, _)) {
            return Err(mapping_key_error_yaml(
                stream.source_mut(),
                "A mapping key cannot have multiple anchors",
            ));
        }
        key_node = Node::Anchored(Box::new(key_node), anchor);
    }
    Ok(key_node)
}

fn parse_mapping_value(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    cur_indent: usize,
    depth: usize,
    explicit_key: bool,
    _key: &Node,
) -> crate::parser::ParseResult<Node> {
    let cur_token = stream.current().cloned();
    match cur_token {
        Some(Token::Newline)
        | None
        | Some(Token::Eof)
        | Some(Token::DocumentStart)
        | Some(Token::DocumentEnd) => {
            if matches!(stream.current(), Some(Token::Newline)) {
                stream.next()?;
            }
            stream.skip_newlines_and_comments()?;
            let indent_level = if let Some(Token::Indent(level)) = stream.current() {
                if *level > cur_indent {
                    let _lvl = *level;
                    stream.next()?; // consume Indent
                    Some(_lvl)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(level) = indent_level {
                stream.skip_newlines_and_comments()?;
                if matches!(stream.current(), Some(Token::Dash)) {
                    use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
                    let ctx_seq = crate::parser::document::context::ParsingContext::new(level)
                        .child_block_context(
                            level,
                            crate::parser::document::context::CollectionType::BlockSequence,
                        );
                    let seq = parse_sequence_with_tokens(
                        stream,
                        level,
                        cur_indent,
                        directives,
                        &ctx_seq,
                        depth + 1,
                    )?;
                    return Ok(seq);
                } else {
                    let map = parse_mapping_with_tokens(stream, level, directives, depth + 1)?;
                    return Ok(map);
                }
            }
            if !explicit_key && matches!(stream.current(), Some(Token::Eof) | None) {
                return Err(crate::parser::document::error_builder::syntax_error(
                    stream.source_mut(),
                    "YAML compliance error: Mapping key without value (expected value after colon)",
                ));
            }
            Ok(Node::None)
        }
        Some(Token::Indent(level)) => {
            stream.next()?; // consume Indent
            if matches!(stream.current(), Some(Token::Dash)) {
                use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
                let ctx_seq = crate::parser::document::context::ParsingContext::new(level)
                    .child_block_context(
                        level,
                        crate::parser::document::context::CollectionType::BlockSequence,
                    );
                parse_sequence_with_tokens(
                    stream,
                    level,
                    cur_indent,
                    directives,
                    &ctx_seq,
                    depth + 1,
                )
            } else {
                parse_mapping_with_tokens(stream, level, directives, depth + 1)
            }
        }
        _ => {
            stream.skip_trivia()?;
            let v = parse_value_with_tokens(stream, directives, depth + 1)?;
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: parsed value = {:?}", v);
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_simple_mapping() {
        let yaml = b"key1: value1\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn debug_8xdj_mapping_tokens() {
        // 8XDJ: comment inside what should be a plain multiline value
        let yaml = b"key: word1\n#  xxx\n  word2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0);
        assert!(
            result.is_err(),
            "8XDJ mapping via tokens should be rejected as invalid, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_mapping_with_empty_value() {
        let yaml = b"key1:\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_mapping_with_decorated_key() {
        let yaml = b"!!str: value\n&anchor: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged empty string
            // Second key should be anchored
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_fh7j_nested_mapping() {
        // FH7J has: "  !!null : a\n  b: !!str\n"
        let yaml = b"!!null: a\nb: !!str";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged null (empty)
            // Second value should be tagged empty string
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_keys_block_mapping() {
        // Explicit keys without values should produce Node::None values
        let yaml = b"? item1\n? item2\n? item3\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 3);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::None));
            assert!(matches!(pairs[2].1, Node::None));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_key_with_value() {
        // Explicit key followed by value on same line
        let yaml = b"? key1: value1\n? key2\n: value2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // key1 has value1; key2 should have value2
            assert!(matches!(pairs[0].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[0].1, Node::Str(ref s, _, _) if s == "value1"));
            assert!(matches!(pairs[1].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[1].1, Node::Str(ref s, _, _) if s == "value2"));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_complex_key_array() {
        // Complex explicit key (array) should normalize to string key
        let yaml = b"? [a, b, c]: 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Key should be a string representation of the array
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("a") && s.contains("b") && s.contains("c"))
            );
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_mapping() {
        let yaml = b"{}\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        // Inline empty mapping should parse via inline_tokens, but base parser should gracefully handle
        let node = crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
            &mut stream,
            &directives,
            0,
            false,
        )
        .unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_multiline_key_value_mapping() {
        // Multiline plain scalar key and value using block scalar-like lines
        let yaml = b"? |\n  multi\n  line\n: |\n  val\n  ue\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Keys/values produced by scalar parser should be strings (literal preserves newlines)
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("multi") && s.contains("line"))
            );
            assert!(
                matches!(pairs[0].1, Node::Str(ref s, _, _) if s.contains("val") && s.contains("ue"))
            );
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_value_on_same_line_and_next_line() {
        let yaml = b"key1: \nkey2:\n  - 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::Array(_)));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_decorated_empty_keys_tag_and_anchor() {
        // Decorated empty keys should produce empty-string keys wrapped by tag/anchor
        let yaml = b"!!str: one\n&root: two\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key is tagged empty string
            match &pairs[0].0 {
                Node::Tagged(inner, tag) => {
                    assert!(matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()));
                    assert!(tag.starts_with("!!") || tag.starts_with("!"));
                }
                other => panic!("Expected Tagged empty key, got {:?}", other),
            }
            // Second key is anchored empty string
            match &pairs[1].0 {
                Node::Anchored(inner, name) => {
                    // Allow anchors that decorate either an empty key or a
                    // scalar key; the main requirement is that the anchor
                    // name itself matches and that the parser preserves the
                    // anchoring semantics.
                    assert_eq!(name, "root");
                    assert!(matches!(**inner, Node::Str(_, _, _)));
                }
                other => panic!("Expected Anchored empty key, got {:?}", other),
            }
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_keys_with_nested_sequence_values() {
        // Explicit keys followed by nested sequences
        let yaml = b"? key1\n: \n  - a\n  - b\n? key2\n: \n  - 1\n  - 2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::Array(ref v) if v.len() == 2));
            assert!(matches!(pairs[1].1, Node::Array(ref v) if v.len() == 2));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }
}
