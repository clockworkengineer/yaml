impl MappingParseContext {
    /// Parse a single key-value pair within a mapping.
    /// Handles explicit keys, omitted values, and YAML edge cases.
    fn parse_mapping_pair(
        &self,
        stream: &mut TokenStream,
        directives: &DirectiveContext,
        cur_indent: usize,
        depth: usize,
    ) -> crate::parser::ParseResult<(Node, Node)> {
        // Indentation validation is now handled in the main loop before calling this function.
        // (Reverted) U44R plain-key indent guard removed.
        #[cfg(feature = "debug-trace")]
        mapping_log(format!(
            "parse_mapping_pair: start, token = {:?}",
            stream.current()
        ));
        #[cfg(feature = "debug-trace")]
        log::debug!("mapping_pair: start at token = {:?}", stream.current());

        let key_start_line = stream.source_line();
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
            // 7LBH: An implicit block mapping key that is a double-quoted scalar
            // spanning multiple source lines is invalid (YAML spec §8.1.1).
            // We detect it here, at the point where we confirm there IS a ':'
            // separator (so the token really is a key, not a value).
            // stream.last_token() holds the DoubleQuoted token since
            // skip_newlines_and_comments() was a no-op (Colon is cur).
            if !explicit_key {
                if let Some(Token::DoubleQuoted(_, true, _)) = stream.last_token() {
                    return Err("Implicit block mapping key cannot span multiple lines \
                         (double-quoted scalar with literal newlines used as implicit key)"
                        .to_string()
                        .into());
                }
                // D49Q: A single-quoted scalar spanning multiple source lines
                // (i.e. the content contains a literal newline) is also invalid
                // as an implicit block mapping key (YAML spec §8.1.1).
                if let Some(Token::SingleQuoted(s)) = stream.last_token() {
                    if s.contains('\n') {
                        return Err("Implicit block mapping key cannot span multiple lines \
                             (single-quoted scalar with literal newlines used as implicit key)"
                            .to_string()
                            .into());
                    }
                }
                // C2SP: A flow collection (sequence or mapping) spanning multiple
                // source lines used as an implicit block mapping key is invalid
                // (YAML spec §8.1.1 — implicit keys must be single-line).
                // We detect this by comparing the raw source-line counter recorded
                // before parsing the key against the counter now.  Unlike
                // `current_line()`, `source_line()` tracks newlines inside flow
                // collections (where `Token::Newline` is suppressed), so it fires
                // correctly for cases like `[23\n]: 42`.
                if stream.source_line() > key_start_line {
                    if matches!(
                        key,
                        crate::nodes::node::Node::Array(_) | crate::nodes::node::Node::Mapping(_)
                    ) {
                        return Err("Implicit block mapping key cannot span multiple lines \
                             (flow collection used as implicit key spans multiple source lines)"
                            .to_string()
                            .into());
                    }
                }
            }
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
            // GDY7: An implicit key (no leading `?`) that reaches EOF without
            // a `:` separator is a YAML violation — but only when:
            //   1. The key is non-empty (not an already-null placeholder).
            //   2. The key sits at the same (or lower) indentation as the base
            //      of the current mapping frame (ruling out indented continuation
            //      content that the value parser left behind in the stream — those
            //      appear at a *higher* indent than the mapping's own base).
            //   3. This mapping frame already has at least one completed pair,
            //      which confirms we are inside an established block mapping and
            //      not at the top of a fresh nested block whose content simply
            //      happens to have no colons (e.g. NB6Z-style scalar blocks).
            if !explicit_key
                && !matches!(key, Node::None)
                && stream.line_indent() <= cur_indent
                && self
                    .stack
                    .last()
                    .map_or(false, |(_, pairs)| !pairs.is_empty())
            {
                return Err(
                    crate::parser::errors::mapping_errors::implicit_key_without_colon_at_eof(
                        stream,
                    )
                    .to_string()
                    .into(),
                );
            }
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
            // G7JE: An implicit block mapping key followed immediately by deeper-indented
            // content (with no `:` separator on the key's own line) is invalid.
            // YAML §8.1.1: implicit keys must be single-line scalars terminated by `: ` on
            // the same line. Detection: `parse_plain_scalar` breaks at the
            // `Indent(level > 0) → peek=Colon` branch, leaving:
            //   - stream.current() = Token::Plain (the nested key name)
            //   - stream.last_token() = Token::Indent(level)  ← unique to this branch
            //   - stream.line_indent() = level > cur_indent
            // Requiring last_token == Indent guards against false positives where
            // parse_plain_scalar exits via a different path and line_indent() happens
            // to be non-zero from a prior context.
            if !explicit_key
                && matches!(cur, Some(Token::Plain(_)))
                && !matches!(key, Node::None)
                && matches!(stream.last_token(), Some(Token::Indent(_)))
                && stream.line_indent() > cur_indent
            {
                return Err("Implicit block mapping key cannot span multiple lines \
                     (implicit key with no colon on its line followed by \
                     deeper-indented content, YAML \u{00a7}8.1.1)"
                    .to_string()
                    .into());
            }
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
            let err =
                crate::parser::errors::mapping_errors::invalid_anchored_alias_key_on_alias_nodes(
                    stream,
                );
            return Err(err.to_string().into());
        }
        if matches!(key_node, Node::Anchored(_, _)) {
            let err =
                crate::parser::errors::mapping_errors::multiple_anchors_on_mapping_key(stream);
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
            // 236B: Only treat the indented block as a value when its indentation
            // exceeds the current mapping level.  A lower-level Indent token means
            // the content belongs to the enclosing scope — it is NOT a nested value
            // for this key.  Leave the token in the stream so the outer mapping loop
            // can handle the dedent correctly.
            if level < cur_indent {
                return Ok(Node::None);
            }
            stream.next()?; // consume Indent (genuinely indented value follows)
            if matches!(stream.current(), Some(Token::Dash)) {
                use crate::parser::tokens::sequence::parse_block_sequence_at;
                return parse_block_sequence_at(stream, level, cur_indent, directives, depth + 1);
            } else {
                return parse_mapping_with_tokens(stream, level, directives, depth + 1);
            }
        }
        _ => {
            // 5U3A: A block sequence indicator '-' cannot appear on the same line
            // as an implicit mapping value colon (YAML spec §8.2.1).
            // We check `cur_token` (before skip_trivia) so that only an
            // IMMEDIATELY-following Dash triggers the error; a Dash that appears
            // after a comment/newline/indent is on a different (valid) indented line.
            // Explicit key-value pairs (e.g. `? - a\n  : - b` in KK5P) are exempt
            // because YAML allows block sequences as both explicit keys and values.
            if !explicit_key && matches!(cur_token, Some(Token::Dash)) {
                return Err(crate::parser::errors::mapping_errors::
                    block_sequence_inline_on_mapping_value_line(stream));
            }
            stream.skip_trivia()?;
            // DK95/01: A double-quoted scalar whose continuation lines are indented
            // at or below the enclosing block mapping's indent level is invalid.
            // (YAML spec §6.1: only spaces count as indentation; tabs do not.)
            // We check min_cont_spaces BEFORE consuming the token to produce a clear error.
            let dq_under_indent = if let Some(Token::DoubleQuoted(_, _, min_spc)) = stream.current()
            {
                let s = *min_spc;
                s <= cur_indent // usize::MAX (no continuation) never triggers (> any cur_indent)
            } else {
                false
            };
            if dq_under_indent {
                return Err(
                    crate::parser::errors::token_errors::tab_as_indentation_in_double_quoted(
                        stream.source_mut(),
                    ),
                );
            }
            // VJP3/00: if the value is immediately a flow mapping (no decorators),
            // pass the current block indent so the inline mapping parser can detect
            // content at the outer block's indentation level (YAML spec violation).
            let v = if matches!(stream.current(), Some(Token::FlowMappingStart)) {
                use crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens;
                parse_inline_mapping_with_tokens(
                    stream,
                    directives,
                    depth + 1,
                    false,
                    Some(cur_indent),
                )?
            } else if matches!(stream.current(), Some(Token::FlowSequenceStart)) {
                // 9C9N: same outer-block-indent guard as for flow mappings — reject
                // flow sequence continuation lines at or below the block mapping indent.
                use crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens;
                parse_inline_sequence_with_tokens(stream, directives, depth + 1, Some(cur_indent))?
            } else {
                parse_value_with_tokens(stream, directives, depth + 1)?
            };
            // Low-hanging fix (Q4CL): After a quoted scalar value, disallow trailing plain text on the same line.
            if let Node::Str(_, quote, _) = &v {
                use crate::nodes::node::QuoteType;
                match quote {
                    QuoteType::Single | QuoteType::Double => {
                        if matches!(stream.current(), Some(Token::Plain(_))) {
                            return Err(crate::parser::errors::mapping_errors::invalid_trailing_plain_text_after_quoted_scalar(stream));
                        }
                        // Re-enable scoped nested ':' guard for quoted scalars:
                        // If a ':' appears immediately after the quoted value on the same
                        // logical line and is followed by same-line plain content, reject.
                        if stream.is_colon_on_same_line() {
                            if matches!(stream.peek()?, Some(Token::Plain(_))) {
                                return Err(crate::parser::errors::mapping_errors::nested_key_separator_in_block_value_same_line(stream));
                            }
                        }
                    }
                    QuoteType::Unquoted => {
                        // ZCZ6: `a: b: c: d` — a plain scalar value immediately
                        // followed by ':' on the same line is ambiguous and invalid
                        // in block context (YAML spec §7.3.3 / §8.1.1).
                        // The lexer already stops a plain scalar at ': ', so `b`
                        // becomes a separate Plain token; we detect the trailing ':'
                        // here and reject rather than silently building a nested map.
                        //
                        // Guard: only fire for BlockStyle::None (true plain scalars).
                        // Block scalars (Folded/Literal) may appear with a trailing ':'
                        // in the token stream due to a separate bug in block scalar
                        // boundary detection; those cases are NOT the ZCZ6 pattern and
                        // must not be rejected here.
                        if matches!(v, Node::Str(_, _, BlockStyle::None)) {
                            if stream.is_colon_on_same_line() {
                                if matches!(stream.peek()?, Some(Token::Plain(_))) {
                                    // ZCZ6 guard: only reject when the key is a non-empty scalar
                                    // (a real implicit mapping key). When the key is empty (from
                                    // a `Colon`-token used as an explicit-value indicator, e.g.
                                    // `  : moon: white` in V9D5), allow the value `moon: white`
                                    // to be parsed as a compact inline mapping by the outer loop.
                                    let key_is_nonempty = matches!(_key,
                                        Node::Str(s, _, _) if !s.is_empty());
                                    if key_is_nonempty {
                                        return Err(crate::parser::errors::mapping_errors::nested_key_separator_in_block_value_same_line(stream));
                                    }
                                }
                            }
                        }
                    }
                }
                // Note: nested ':' cases are validated elsewhere to avoid false positives with
                // multi-line/block scalar values and flow contexts.
            }
            // Note: Nested ':' after a value on the same line is handled in downstream parsing.
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: parsed value = {:?}", v);
            Ok(v)
        }
    }
}
