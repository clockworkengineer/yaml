//! Value Token Parsing
//!
//! Implements token-based parsing for YAML values, demonstrating how tokenization
//! solves decorator parsing problems and eliminates infinite loops.
//!
//! This module is split into:
//!   coerce.rs — tag coercion helpers (should_preserve_double_bang, pick_tag_text, try_coerce_tag)
//!
//! Copyright (c) 2026 YAML Library Developers

// ...existing code...
use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::directives::DirectiveContext;
const MAX_NESTING_DEPTH: usize = 128;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Parse a value using tokens (proof of concept)
///
/// This demonstrates how tokenization solves the decorator problem:
/// 1. Decorators are consumed upfront without complex lookahead
/// 2. No infinite loops - tokens have clear boundaries
/// 3. Empty values are naturally supported (EOF after decorator)
/// 4. Both tag+anchor and anchor+tag orderings work
pub fn parse_value_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    if depth > MAX_NESTING_DEPTH {
        return Err(
            crate::parser::errors::token_errors::unexpected_token_in_value(
                stream.source_mut(),
                &Token::Plain("Nesting too deep: possible malicious or malformed YAML".to_string()),
            ),
        );
    }
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "value_tokens: ENTER parse_value_with_tokens (depth {}), current token: {:?}",
        depth,
        stream.current()
    );
    // Handle aliases first (they don't have content)
    if matches!(stream.current(), Some(Token::Alias(_))) {
        if let Some(Token::Alias(name)) = stream.current() {
            let alias = name.clone();
            stream.next()?;
            #[cfg(feature = "debug-trace")]
            log::debug!("value_tokens: parsed alias = {}", alias);
            // Strict: alias must resolve, otherwise error
            if alias.is_empty() {
                return Err(crate::parser::errors::token_errors::empty_token_name(
                    stream.source_mut(),
                    "Empty alias name is not allowed",
                ));
            }
            return Ok(Node::Alias(alias));
        }
    }

    // Consume decorators using unified helper (NO INFINITE LOOPS!)
    let decorators = stream.consume_decorators()?;

    // If we have decorators, parse the content
    if decorators.tag.is_some() || decorators.anchor.is_some() {
        #[cfg(feature = "debug-trace")]
        log::debug!("value_tokens: decorators = {:?}", decorators);
        // Do NOT skip indentation/newlines here. Indentation signals nested
        // block structures (e.g., a mapping following a tag like !!set), and
        // consuming it would hide structure boundaries from the value parser.
        // Only skip comments; structural whitespace must be preserved.
        stream.skip_comments()?;

        // Special handling: if tag is !!seq, parse a sequence (block or flow)
        let tag_is_seq = decorators
            .tag
            .as_ref()
            .map(|t| crate::parser::utils::node_utils::resolved_is_seq(&directives.resolve_tag(t)))
            .unwrap_or(false);

        if tag_is_seq {
            // In flow context, a tagged sequence may be written as
            // `!!seq [ a, b ]` but split across lines, e.g. in EHF6:
            // `k: !!seq` then on the next indented line `[ a, !!str b ]`.
            // Inside a flow collection, indentation is just horizontal
            // whitespace, so we can safely skip Indent tokens here and
            // detect a following flow sequence start.
            if stream.current_flow_depth() > 0 {
                // Skip any indentation directly after the tag
                while matches!(stream.current(), Some(Token::Indent(_))) {
                    stream.next()?;
                }

                if matches!(stream.current(), Some(Token::FlowSequenceStart)) {
                    use crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens;
                    let seq =
                        parse_inline_sequence_with_tokens(stream, directives, depth + 1, None)?;
                    let mut result =
                        Node::Tagged(Box::new(seq), "tag:yaml.org,2002:seq".to_string());
                    if let Some(anchor_name) = decorators.anchor {
                        result = Node::Anchored(Box::new(result), anchor_name);
                    }
                    return Ok(result);
                }
            }

            // Outside flow, an increased indentation after !!seq indicates
            // a block sequence value (e.g. `!!seq` followed by `- a`).
            if let Some(Token::Indent(level)) = stream.current() {
                use crate::parser::tokens::sequence::parse_block_sequence_at;
                let seq = parse_block_sequence_at(stream, *level, 0, directives, depth + 1)?;
                let mut result = Node::Tagged(Box::new(seq), "tag:yaml.org,2002:seq".to_string());
                if let Some(anchor_name) = decorators.anchor {
                    result = Node::Anchored(Box::new(result), anchor_name);
                }
                return Ok(result);
            }
        }

        // NOW check if we're at EOF or end of structure (empty decorated value)
        // This includes:
        // - EOF/None: end of document
        // - Dash: next sequence item (e.g., "- !!str\n-" = empty tagged value)
        // - Colon: mapping key (e.g., "!!str :")
        // - Comma: next entry in a flow collection (e.g., "foo: !!str,")
        // - FlowMappingEnd/FlowSequenceEnd: end of flow collection
        // - DocumentStart/DocumentEnd: document boundary
        // SY6V: Disallow an anchor immediately preceding a block sequence indicator ('-').
        // "&anchor - item" is invalid; anchors must attach to the node being introduced,
        // e.g., "- &anchor item". Treat this as a syntax error rather than an empty
        // decorated value when not inside a flow collection.
        if matches!(stream.current(), Some(Token::Dash))
            && decorators.anchor.is_some()
            && stream.current_flow_depth() == 0
        {
            return Err(crate::parser::errors::anchor_errors::AnchorErrors::anchor_cannot_precede_dash_block(stream));
        }
        // U99R: Disallow a comma immediately after a tag in block context.
        // In block (non-flow) context, ',' is not a valid value separator.
        // "- !!str, xxx" should be a syntax error, not an empty value.
        if matches!(stream.current(), Some(Token::Comma))
            && stream.current_flow_depth() == 0
            && match decorators.tag.as_ref() {
                Some(t) => !t.starts_with("!<"),
                None => true,
            }
        {
            return Err(
                crate::parser::errors::token_errors::unexpected_comma_after_tag_in_block_value(
                    stream.source_mut(),
                ),
            );
        }
        match stream.current() {
            Some(Token::Eof)
            | Some(Token::Dash)
            | Some(Token::Colon)
            | Some(Token::Comma)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd)
            | Some(Token::DocumentStart)
            | Some(Token::DocumentEnd)
            | None => {
                // Decorator with no content - empty value
                let mut result = Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None);

                if let Some(tag_raw_ref) = decorators.tag.as_ref() {
                    // QLJ7: If the tag uses an explicit handle (e.g., !prefix!Type),
                    // require that the handle was defined via %TAG in this document.
                    // Otherwise, produce a parse error rather than falling back to
                    // a local tag resolution.
                    directives
                        .validate_tag_handle_usage(tag_raw_ref)
                        .map_err(|e| {
                            crate::parser::errors::token_errors::invalid_tag_handle_usage(
                                stream.source_mut(),
                                &e.to_string(),
                            )
                        })?;
                    let resolved = directives.resolve_tag(tag_raw_ref);
                    if let Some(coerced) = try_coerce_tag(&resolved, result.clone()) {
                        result = coerced;
                    } else {
                        let tag_out = pick_tag_text(tag_raw_ref, resolved);
                        result = Node::Tagged(Box::new(result), tag_out);
                    }
                }

                if let Some(anchor_name) = decorators.anchor {
                    result = Node::Anchored(Box::new(result), anchor_name);
                }

                #[cfg(feature = "debug-trace")]
                log::debug!("value_tokens: empty decorated value -> {:?}", result);
                return Ok(result);
            }
            _ => {
                // There's content after decorators, continue to parse it
            }
        }

        // Special handling: if tag is !!set and next token is FlowMappingStart, parse as set
        let tag_is_set = decorators
            .tag
            .as_ref()
            .map(|t| crate::parser::utils::node_utils::resolved_is_set(&directives.resolve_tag(t)))
            .unwrap_or(false);

        // SY6V: Disallow an anchor immediately followed by a plain token that starts
        // with a block sequence indicator ('-') in block context. This pattern
        // ("&anchor - item") is invalid; the anchor must attach to the node being
        // introduced ("- &anchor item").
        if decorators.anchor.is_some()
            && stream.current_flow_depth() == 0
            && matches!(stream.current(), Some(Token::Plain(s)) if s.trim_start().starts_with('-'))
        {
            return Err(
                crate::parser::errors::anchor_errors::AnchorErrors::anchor_cannot_precede_dash_block(
                    stream,
                ),
            );
        }

        let mut result = if tag_is_set {
            if matches!(stream.current(), Some(Token::FlowMappingStart)) {
                // Use set mode for inline mapping
                crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
                    stream,
                    directives,
                    depth + 1,
                    true,
                    None,
                )?
            } else {
                parse_value_content(stream, directives, depth + 1)?
            }
        } else {
            parse_value_content(stream, directives, depth + 1)?
        };

        if let Some(tag_raw_ref) = decorators.tag.as_ref() {
            // QLJ7: Enforce that explicit handles are declared via %TAG
            directives
                .validate_tag_handle_usage(tag_raw_ref)
                .map_err(|e| {
                    crate::parser::errors::token_errors::invalid_tag_handle_usage(
                        stream.source_mut(),
                        &e.to_string(),
                    )
                })?;
            let tag_resolved = directives.resolve_tag(tag_raw_ref);
            if tag_resolved == "!!str"
                || tag_resolved == "!str"
                || tag_resolved == "tag:yaml.org,2002:str"
            {
                // For block scalars, preserve block style; otherwise, coerce to plain string
                match &result {
                    Node::Str(s, q, style)
                        if style == &BlockStyle::Literal || style == &BlockStyle::Folded =>
                    {
                        result = Node::Str(s.clone(), q.clone(), style.clone());
                    }
                    Node::Str(s, _, _) => {
                        result = Node::Str(s.clone(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::Number(Numeric::Integer(i)) => {
                        result = Node::Str(i.to_string(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::Number(Numeric::Float(f)) => {
                        result = Node::Str(f.to_string(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::Boolean(b) => {
                        result = Node::Str(b.to_string(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::None => {
                        result = Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    _ => {
                        result = Node::Str(
                            format!("{:?}", result),
                            QuoteType::Unquoted,
                            BlockStyle::None,
                        );
                    }
                }
            } else if crate::parser::utils::node_utils::resolved_is_seq(&tag_resolved) {
                // Always wrap as Tagged with canonical tag for sequences
                match &result {
                    Node::Array(items) => {
                        result = Node::Tagged(
                            Box::new(Node::Array(items.clone())),
                            "tag:yaml.org,2002:seq".to_string(),
                        );
                    }
                    _ => {
                        // If not an array, still wrap whatever is there
                        result =
                            Node::Tagged(Box::new(result), "tag:yaml.org,2002:seq".to_string());
                    }
                }
            } else if crate::parser::utils::node_utils::resolved_is_map(&tag_resolved) {
                // Always wrap as Tagged with canonical tag for mappings
                match &result {
                    Node::Mapping(pairs) => {
                        result = Node::Tagged(
                            Box::new(Node::Mapping(pairs.clone())),
                            "tag:yaml.org,2002:map".to_string(),
                        );
                    }
                    _ => {
                        result =
                            Node::Tagged(Box::new(result), "tag:yaml.org,2002:map".to_string());
                    }
                }
            } else if let Some(coerced) = try_coerce_tag(&tag_resolved, result.clone()) {
                result = coerced;
            } else {
                let tag_out = match decorators.tag.as_ref() {
                    Some(tag_raw_ref) => pick_tag_text(tag_raw_ref, tag_resolved),
                    None => tag_resolved,
                };
                result = Node::Tagged(Box::new(result), tag_out);
            }
        }

        if let Some(anchor_name) = decorators.anchor {
            // SR86 / SU74: YAML 1.2 does not allow anchors to be applied
            // directly to alias nodes (e.g., "&b *a" or "&b *alias : v").
            // If the decorated value resolved to an alias, treat this as a
            // structural error rather than accepting an anchored alias.
            if matches!(result, Node::Alias(_)) {
                return Err(
                    crate::parser::errors::anchor_errors::AnchorErrors::invalid_anchored_alias(
                        stream,
                    ),
                );
            }
            if matches!(result, Node::Anchored(_, _)) {
                return Err(
                    crate::parser::errors::anchor_errors::AnchorErrors::multiple_anchors(stream),
                );
            }
            result = Node::Anchored(Box::new(result), anchor_name);
        }

        #[cfg(feature = "debug-trace")]
        log::debug!("value_tokens: decorated value -> {:?}", result);
        return Ok(result);
    }

    // No decorators - parse plain value
    let node = parse_value_content(stream, directives, depth + 1);
    #[cfg(feature = "debug-trace")]
    if let Ok(ref n) = node {
        log::debug!("value_tokens: plain value -> {:?}", n);
    }
    node
}

/// Parse value content (the actual value after decorators)
fn parse_value_content(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "value_tokens: parse_value_content at token = {:?}",
        stream.current()
    );
    // Skip comments before parsing value (DRY)
    stream.skip_comments()?;
    match stream.current() {
        // Tolerate stray commas in block contexts: consume and treat as empty value
        Some(Token::Comma) => {
            stream.next()?;
            stream.skip_trivia()?;
            return Ok(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ));
        }
        Some(Token::FlowMappingStart) => {
            use crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens;
            parse_inline_mapping_with_tokens(stream, directives, depth + 1, false, None)
        }
        Some(Token::FlowSequenceStart) => {
            use crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens;
            parse_inline_sequence_with_tokens(stream, directives, depth + 1, None)
        }
        Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(..)) | Some(Token::Plain(_)) => {
            crate::parser::document::scalar::parse_scalar_with_tokens(stream, directives, 0)
        }
        Some(Token::Dash) => {
            use crate::parser::tokens::sequence::parse_block_sequence_at;
            parse_block_sequence_at(stream, 0, 0, directives, depth + 1)
        }
        Some(Token::Indent(level)) => {
            // Indented value: parse nested mapping
            use crate::parser::tokens::mapping::parse_mapping_with_tokens;
            parse_mapping_with_tokens(stream, *level, directives, depth + 1)
        }
        Some(Token::Newline) => {
            // Handle newline before an indented block value. If there's an increased indent
            // after the newline, parse a nested mapping or sequence as the value.
            // Save source state before consuming the Newline: the lexer's look-ahead will
            // scan the next token (possibly Token::DocumentStart from '---'), advancing the
            // underlying source past those marker characters. If we end up returning an
            // empty value because there is no indented block after the newline, we must
            // restore source so the outer parser can detect the document boundary.
            let pre_newline_state = stream.source_mut().save_state();
            stream.next()?; // consume Newline
            // Check for indentation increase
            if let Some(Token::Indent(level)) = stream.current() {
                if *level > 0 {
                    let _lvl = *level;
                    stream.next()?; // consume Indent
                    // Skip subsequent newlines/comments
                    stream.skip_newlines_and_comments()?;
                    // Decide between sequence or mapping based on next token
                    if matches!(stream.current(), Some(Token::Dash)) {
                        use crate::parser::tokens::sequence::parse_block_sequence_at;
                        return parse_block_sequence_at(stream, _lvl, 0, directives, depth + 1);
                    } else {
                        // 4JVG: If the next token is a decorator (Anchor/Tag), probe
                        // whether a colon follows the first value token.  When no
                        // colon is found, the indented content is a decorated scalar
                        // (not a mapping key), so route to parse_value_with_tokens.
                        // This lets the caller's existing `multiple_anchors` check
                        // fire when two anchors refer to the same scalar value.
                        if matches!(
                            stream.current(),
                            Some(Token::Anchor(_)) | Some(Token::Tag(_))
                        ) && !stream.probe_has_colon_after_decorator_and_value()
                        {
                            return parse_value_with_tokens(stream, directives, depth + 1);
                        }
                        use crate::parser::tokens::mapping::parse_mapping_with_tokens;
                        return parse_mapping_with_tokens(stream, _lvl, directives, depth + 1);
                    }
                }
            }
            // No indentation increase: if we are at a DocumentStart marker (---)
            // or at EOF (None), restore source so the outer parser can detect the
            // document boundary or EOF correctly.
            // Note: DocumentEnd (...) is intentionally NOT restored here.
            if matches!(stream.current(), Some(Token::DocumentStart) | None) {
                stream.source_mut().restore_state(pre_newline_state);
            }
            // Treat as empty string value
            Ok(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ))
        }
        Some(Token::Eof) | None => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(Token::Alias(name)) => {
            let alias_name = name.clone();
            stream.next()?;
            Ok(Node::Alias(alias_name))
        }
        Some(Token::Colon) => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(Token::QuestionMark) => {
            // Explicit key marker - parse as mapping with explicit keys
            use crate::parser::tokens::mapping::parse_mapping_with_tokens;
            // Parse mapping at indent 0 (explicit keys can appear at any indent)
            parse_mapping_with_tokens(stream, 0, directives, depth + 1)
        }
        Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd) => {
            // Flow collection end with no value = implicit null/empty
            // e.g., {key:} or [item,]
            Ok(Node::None)
        }
        Some(token) => {
            let tok = token.clone();
            #[cfg(feature = "debug-trace")]
            log::debug!(
                "value_tokens: error -> Unexpected token in value: {:?}",
                tok
            );
            Err(
                crate::parser::errors::token_errors::unexpected_token_in_value(
                    stream.source_mut(),
                    &tok,
                ),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_tag_on_empty_value() {
        // This is the FH7J pattern that caused infinite loops!
        let mut source = Buffer::new(b"!!str");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as empty string
        assert!(matches!(result, Node::Str(s, _, _) if s.is_empty()));
    }

    #[test]
    fn test_anchor_on_empty_value() {
        // This is the PW8X pattern that caused infinite loops!
        let mut source = Buffer::new(b"&anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as anchored empty string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_both_decorators_on_empty() {
        let mut source = Buffer::new(b"!!str &anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as anchored empty string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_tag_with_plain_value() {
        let mut source = Buffer::new(b"!!str hello");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as string "hello"
        assert!(matches!(result, Node::Str(s, _, _) if s == "hello"));
    }

    #[test]
    fn test_anchor_with_quoted_value() {
        let mut source = Buffer::new(b"&anchor 'hello'");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as anchored string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s == "hello"));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_alias() {
        let mut source = Buffer::new(b"*myalias");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as alias
        assert!(matches!(result, Node::Alias(name) if name == "myalias"));
    }

    #[test]
    fn test_error_on_multiple_anchors_for_single_node() {
        // Two anchors adjacent should error (single-anchor per node)
        let mut source = Buffer::new(b"&a &b 123");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let err = parse_value_with_tokens(&mut stream, &directives, 0).unwrap_err();
        let err_str = err.to_string().to_ascii_lowercase();
        assert!(err_str.contains("duplicate anchor") || err_str.contains("multiple anchors"));
    }

    #[test]
    fn test_tag_followed_by_indented_mapping() {
        // Decorator then indented block value should parse nested mapping
        let mut source = Buffer::new(b"!!set\n  a: null\n  b: null\n");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // !!set should coerce mapping/nulls into a Set
        match result {
            Node::Set(items) => {
                assert_eq!(items.len(), 2);
            }
            other => panic!("Expected Set, got {:?}", other),
        }
    }

    #[test]
    fn test_anchor_followed_by_indented_mapping() {
        // Anchor then indented block value should wrap nested mapping in Anchored
        let mut source = Buffer::new(b"&root\n  key: value\n");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "root");
                assert!(matches!(*inner, Node::Mapping(_)));
            }
            _ => panic!("Expected Anchored mapping"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tag coercion helpers — included directly into this module.
// ---------------------------------------------------------------------------
include!("coerce.rs");
