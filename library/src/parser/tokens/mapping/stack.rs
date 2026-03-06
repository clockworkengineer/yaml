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
        _depth: usize,
    ) -> crate::parser::ParseResult<Option<Node>> {
        match token {
            Some(Token::Indent(level)) if *level < current_indent => {
                // When we see an indent less than current, we should exit this mapping.
                // The validity of the indent level will be checked by the parent mapping.
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
                crate::parser::utils::helpers::validate_trailing_content_after_document_end(
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
        // Early U44R guard: if the raw source indent indicates a dedent below
        // both the current indent and the base indent, reject before attempting
        // to parse the next pair. This catches cases where no explicit Indent
        // token is emitted by the tokenizer.
        // (Reverted) U44R early raw-indent guard removed to preserve
        // existing integration behavior; YAML suite coverage will catch
        // inconsistent indentation at document level when applicable.
        let token = stream.current().cloned();
        if let Some(result) = self.handle_indent_tokens(
            stream,
            token.clone(),
            current_indent,
            saw_comment_between_entries,
            depth,
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
        _depth: usize,
    ) -> crate::parser::ParseResult<Option<Option<(Node, Node)>>> {
        if let Some(Token::Indent(level)) = token {
            let last_value_is_empty = self
                .stack
                .last()
                .and_then(|(_, pairs)| pairs.last())
                .map(|(_, v)| matches!(v, Node::None))
                .unwrap_or(false);
            if level > current_indent {
                // 8XDJ guard: If a standalone comment appeared between entries and
                // the previous value is already complete, an indented block must
                // not extend that value.
                if !last_value_is_empty && saw_comment_between_entries {
                    let err = crate::parser::errors::mapping_errors::
                        invalid_indentation_after_comment_in_mapping_value(stream);
                    return Err(err.to_string().into());
                }
                // Consume the indent token to inspect the next token accurately.
                stream.next()?;
                // Allow entering a nested mapping level only if the previous
                // key has an omitted value (Node::None), which signals that
                // the indented block belongs to that key's value.
                if last_value_is_empty {
                    self.stack.push((level, Vec::new()));
                    return Ok(Some(None));
                }
                // U44R fix (scoped): If indentation increases by one space and the
                // next token begins a plain key followed immediately by ':', treat
                // this as an invalid misaligned key rather than starting a nested block.
                if level == current_indent + 1 {
                    if matches!(stream.current(), Some(Token::Plain(_))) {
                        if let Some(next) = stream.peek()? {
                            if matches!(next, Token::Colon) {
                                let err = crate::parser::errors::mapping_errors::
                                    invalid_indentation_extending_completed_mapping_value(stream);
                                return Err(err.to_string().into());
                            }
                        }
                    }
                }
                // Otherwise, continue parsing at the current level.
                return Ok(Some(None));
            }
            if level < current_indent {
                // (Reverted) U44R shallow dedent guard removed - conflicts with DMG6 fix
                // and valid nested mappings. The DMG6 fix in parse_mapping_loop handles
                // inconsistent indentation errors.
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
