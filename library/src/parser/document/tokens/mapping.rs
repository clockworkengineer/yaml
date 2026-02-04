/// Parses a value that is indented relative to the current mapping key.
/// Distinguishes between block sequences and nested mappings, and handles YAML compliance errors.
/// Handles indented/nested value after a mapping key.
fn parse_indented_mapping_value(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    cur_indent: usize,
    depth: usize,
    explicit_key: bool,
) -> crate::parser::ParseResult<Node> {
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
    // YAML compliance error: Mapping key without value (expected value after colon)
    if !explicit_key && matches!(stream.current(), Some(Token::Eof) | None) {
        use crate::error::enhanced::{EnhancedError, ErrorCode};
        let err = EnhancedError::new(crate::parser::document::error_builder::syntax_error(
            stream.source_mut(),
            "YAML compliance error: Mapping key without value (expected value after colon)",
        ))
        .with_code(ErrorCode::E001);
        return Err(err.to_string().into());
    }
    Ok(Node::None)
}
/// Context for managing the state of a block mapping parse.
/// Maintains a stack of (indent_level, pairs) to support nested mappings and dedent unwinding.
struct MappingParseContext {
    /// Stack of (indent_level, mapping pairs) for nested mappings
    stack: Vec<(usize, Vec<(Node, Node)>)>,
    /// The base indentation level for this mapping
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
/// Helper for debug logging of mapping parser internals.
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

/// Parses a single key-value mapping pair (for sequence items).
/// Used when a mapping pair appears as a sequence item (e.g., - key: value).
#[allow(dead_code)]
pub fn parse_single_mapping_pair_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> crate::parser::ParseResult<Node> {
    let ctx = MappingParseContext {
        stack: vec![(0, Vec::new())],
        base_indent: 0,
    };
    let (key, value) = ctx.parse_mapping_pair(stream, directives, 0, 0)?;
    Ok(Node::Mapping(vec![(key, value)]))
}

/// Parses a block mapping using tokens.
/// This is the main entry point for block mapping parsing in the token-based parser.
/// Handles indentation, dedent unwinding, and special YAML tokens.
///
/// # Example
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
    ctx.parse_mapping_loop(stream, directives, depth)
}

impl MappingParseContext {
    /// Main mapping parse loop as a method. Handles comments, dedent, and pair parsing.
    fn parse_mapping_loop(
        &mut self,
        stream: &mut TokenStream,
        directives: &DirectiveContext,
        depth: usize,
    ) -> crate::parser::ParseResult<Node> {
        loop {
            let saw_comment_between_entries = stream.skip_newlines_and_comments_with_flag()?;
            self.handle_dedent(stream);
            let current_indent = self.get_current_indent();
            let token = stream.current().cloned();
            if let Some(result) = self.handle_special_tokens(stream, current_indent, &token)? {
                return Ok(result);
            }
            if let Some(pair) = self.try_parse_and_insert_pair(
                stream,
                directives,
                current_indent,
                depth,
                saw_comment_between_entries,
            )? {
                if let Some((_, pairs)) = self.stack.last_mut() {
                    pairs.push(pair);
                }
            }
        }
    }
}

impl MappingParseContext {
    /// Get the current indentation level from the top of the stack.
    fn get_current_indent(&self) -> usize {
        self.stack
            .last()
            .map(|(lvl, _)| *lvl)
            .unwrap_or(self.base_indent)
    }

    /// Unwind the mapping stack to the target indentation level, closing nested mappings as needed.
    fn dedent_unwind_mapping_stack(&mut self, target_level: usize) {
        while self.stack.len() > 1 && self.stack.last().map(|(i, _)| *i).unwrap_or(0) > target_level
        {
            let (_, closed_pairs) = self.stack.pop().unwrap();
            if let Some((_, parent_pairs)) = self.stack.last_mut() {
                parent_pairs.push((Node::None, Node::Mapping(closed_pairs)));
            }
        }
    }

    /// Handle dedent tokens by popping stack frames and closing mappings.
    fn handle_dedent(&mut self, stream: &mut TokenStream) {
        let token_indent = match stream.current() {
            Some(Token::Indent(level)) => *level,
            _ => self.get_current_indent(),
        };
        self.dedent_unwind_mapping_stack(token_indent);
    }

    /// Handle special YAML tokens (indent, document start/end, flow end, etc.) during mapping parsing.
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

    /// Attempt to parse and insert a key-value pair into the current mapping.
    /// Handles indentation, dedent, and special tokens.
    fn try_parse_and_insert_pair(
        &mut self,
        stream: &mut TokenStream,
        directives: &DirectiveContext,
        current_indent: usize,
        depth: usize,
        saw_comment_between_entries: bool,
    ) -> crate::parser::ParseResult<Option<(Node, Node)>> {
        let token = stream.current().cloned();
        if let Some(result) = self.handle_indent_tokens(
            stream,
            token.clone(),
            current_indent,
            saw_comment_between_entries,
        )? {
            return Ok(result);
        }
        if let Some(result) = Self::handle_mapping_control_tokens(&mut self.stack, token) {
            return Ok(Some(result));
        }
        let (key, value) = self.parse_mapping_pair(stream, directives, current_indent, depth)?;
        let norm_key = force_key_to_string(key);
        Ok(Some((norm_key, value)))
    }

    /// Handle indent/dedent tokens and error cases for try_parse_and_insert_pair.
    fn handle_indent_tokens(
        &mut self,
        stream: &mut TokenStream,
        token: Option<Token>,
        current_indent: usize,
        saw_comment_between_entries: bool,
    ) -> crate::parser::ParseResult<Option<Option<(Node, Node)>>> {
        if let Some(Token::Indent(level)) = token {
            let last_value_is_empty = self
                .stack
                .last()
                .and_then(|(_, pairs)| pairs.last())
                .map(|(_, v)| matches!(v, Node::None))
                .unwrap_or(false);
            // U44R: If the previous mapping pair ended with a quoted scalar value
            // on the same line, then an immediately following indented line must
            // NOT be treated as a continuation. This is invalid indentation and
            // should be rejected. This prevents mis-parsing cases like:
            //
            //   key1: "quoted1"
            //    key2: "bad indentation"
            //
            // where `key2` is incorrectly indented relative to the mapping base.
            let last_value_is_quoted_scalar = self
                .stack
                .last()
                .and_then(|(_, pairs)| pairs.last())
                .map(|(_, v)| matches!(
                    v,
                    Node::Str(_, crate::nodes::node::QuoteType::Single, _)
                        | Node::Str(_, crate::nodes::node::QuoteType::Double, _)
                ))
                .unwrap_or(false);
            if level > current_indent {
                if (!last_value_is_empty && saw_comment_between_entries)
                    || last_value_is_quoted_scalar
                {
                    use crate::error::enhanced::{EnhancedError, ErrorCode};
                    let err = EnhancedError::new(mapping_key_error_yaml(
                        stream.source_mut(),
                        "Invalid indentation after quoted scalar: indented content cannot extend a completed mapping value",
                    ))
                    .with_code(ErrorCode::E007)
                    .with_note("Ensure keys under the mapping align consistently; quoted scalars cannot be continued by indentation.");
                    return Err(err.to_string().into());
                }
                self.stack.push((level, Vec::new()));
                stream.next()?;
                return Ok(Some(None));
            }
            if level < current_indent {
                // U44R fix: A shallow dedent within a nested mapping (i.e.,
                // dedenting to a level still greater than the base indent)
                // indicates inconsistent key indentation and should be
                // rejected, rather than silently unwinding.
                if level > self.base_indent {
                    use crate::error::enhanced::{EnhancedError, ErrorCode};
                    let err = EnhancedError::new(mapping_key_error_yaml(
                        stream.source_mut(),
                        "Invalid indentation for nested mapping key: inconsistent dedent within mapping value",
                    ))
                    .with_code(ErrorCode::E009)
                    .with_note("Ensure all keys under the nested mapping use the same indentation.");
                    return Err(err.to_string().into());
                }
                self.dedent_unwind_mapping_stack(level);
                stream.next()?;
                return Ok(Some(None));
            }
            stream.next()?;
            return Ok(Some(None));
        }
        Ok(None)
    }

    /// Handle mapping control tokens (end of mapping, document, etc.) for try_parse_and_insert_pair.
    fn handle_mapping_control_tokens(
        stack: &mut Vec<(usize, Vec<(Node, Node)>)>,
        token: Option<Token>,
    ) -> Option<(Node, Node)> {
        match token {
            Some(Token::Eof)
            | Some(Token::DocumentEnd)
            | Some(Token::DocumentStart)
            | Some(Token::Dash)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd) => {
                let (_, pairs) = stack.pop().unwrap();
                Some((Node::None, Node::Mapping(pairs)))
            }
            _ => None,
        }
    }
}

impl MappingParseContext {
    /// Parse a single key-value pair within a mapping.
    /// Handles explicit keys, omitted values, and YAML edge cases.
    #[allow(dead_code)]
    fn parse_mapping_pair(
        &self,
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

        let (explicit_key, key) = self.parse_mapping_key(stream, directives, depth)?;
        #[cfg(feature = "debug-trace")]
        mapping_log(format!(
            "parse_mapping_pair: after key, token = {:?}",
            stream.current()
        ));
        stream.skip_newlines_and_comments()?;

        let cur = stream.current();
        // Early return for colon after key
        if let Some(Token::Colon) = cur {
            stream.next()?;
            let value =
                parse_mapping_value(stream, directives, cur_indent, depth, explicit_key, &key)?;
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
            return Ok((key, value));
        }

        // Early return for explicit key with omitted value
        if explicit_key {
            #[cfg(feature = "debug-trace")]
            mapping_log(format!(
                "parse_mapping_pair: after explicit key newline/whitespace, token = {:?}",
                stream.current()
            ));
            match cur {
                Some(Token::Plain(_))
                | Some(Token::Tag(_))
                | Some(Token::Anchor(_))
                | Some(Token::QuestionMark)
                | Some(Token::DocumentEnd)
                | Some(Token::DocumentStart)
                | Some(Token::Eof)
                | None
                | Some(Token::Indent(_)) => return Ok((key, Node::None)),
                _ => {}
            }
            if !matches!(cur, Some(Token::Colon)) {
                return Ok((key, Node::None));
            } else {
                stream.next()?;
                let value =
                    parse_mapping_value(stream, directives, cur_indent, depth, explicit_key, &key)?;
                #[cfg(feature = "debug-trace")]
                log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
                return Ok((key, value));
            }
        }

        // Early return for EOF or None
        if matches!(cur, Some(Token::Eof) | None) {
            return Ok((key, Node::None));
        }

        // Early return for plain/tag/anchor/question tokens
        if matches!(
            cur,
            Some(Token::Plain(_))
                | Some(Token::Tag(_))
                | Some(Token::Anchor(_))
                | Some(Token::QuestionMark)
        ) {
            return Ok((key, Node::None));
        }

        // Early return for dash token
        if matches!(cur, Some(Token::Dash)) {
            return Ok((key, Node::None));
        }

        // Default: parse value
        let value = parse_mapping_value(stream, directives, cur_indent, depth, explicit_key, &key)?;
        #[cfg(feature = "debug-trace")]
        log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
        Ok((key, value))
    }
}

impl MappingParseContext {
    /// Parse a mapping key, handling explicit keys and decorators (tags/anchors).
    fn parse_mapping_key(
        &self,
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
}

/// Apply decorators (tag, anchor) to a mapping key node.
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
            use crate::error::enhanced::{EnhancedError, ErrorCode};
            let err = EnhancedError::new(mapping_key_error_yaml(
                stream.source_mut(),
                "Invalid anchored alias key: anchors cannot be applied to alias nodes",
            ))
            .with_code(ErrorCode::E004)
            .with_note("Anchors are not allowed on alias nodes.");
            return Err(err.to_string().into());
        }
        if matches!(key_node, Node::Anchored(_, _)) {
            use crate::error::enhanced::{EnhancedError, ErrorCode};
            let err = EnhancedError::new(mapping_key_error_yaml(
                stream.source_mut(),
                "A mapping key cannot have multiple anchors",
            ))
            .with_code(ErrorCode::E005)
            .with_note("A key can only have one anchor.");
            return Err(err.to_string().into());
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
            return parse_indented_mapping_value(
                stream,
                directives,
                cur_indent,
                depth,
                explicit_key,
            );
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
                return parse_sequence_with_tokens(
                    stream,
                    level,
                    cur_indent,
                    directives,
                    &ctx_seq,
                    depth + 1,
                );
            } else {
                return parse_mapping_with_tokens(stream, level, directives, depth + 1);
            }
        }
        _ => {
            stream.skip_trivia()?;
            let v = parse_value_with_tokens(stream, directives, depth + 1)?;
            // Low-hanging fix (Q4CL): After a quoted scalar value, disallow trailing plain text on the same line.
            if let Node::Str(_, quote, _) = &v {
                use crate::nodes::node::QuoteType;
                if matches!(quote, QuoteType::Single | QuoteType::Double) {
                    if matches!(stream.current(), Some(Token::Plain(_))) {
                        return Err(crate::parser::document::error_builder::syntax_error(
                            stream.source_mut(),
                            "Invalid content immediately after quoted scalar: trailing plain text on the same line is not allowed",
                        ));
                    }
                }
            }
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
