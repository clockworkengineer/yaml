/// Block structure scanning — indentation, dedents, and newline handling.
///
/// All methods here are part of `impl<'a> Lexer<'a>`.  They are included
/// directly into the `lexer` module from `mod.rs` via `include!("block.rs")`,
/// so they live in the same module scope as the rest of the lexer.  Private
/// struct fields are therefore accessible without any visibility changes.
impl<'a> Lexer<'a> {
    /// Utility: Peek next non-whitespace character without consuming
    fn peek_next_non_whitespace(&mut self) -> Option<char> {
        let state = self.source.save_state();
        while let Some(ch) = self.source.current() {
            if ch == CHAR_SPACE || ch == CHAR_TAB {
                self.source.next();
            } else {
                break;
            }
        }
        let result = self.source.current();
        self.source.restore_state(state);
        result
    }

    /// Skip horizontal whitespace (space and tab)
    fn skip_horizontal_whitespace(&mut self) {
        while let Some(ch) = self.source.current() {
            if ch == CHAR_SPACE || ch == CHAR_TAB {
                self.source.next();
            } else {
                break;
            }
        }
    }

    /// Emit an indentation token if applicable at line start.
    /// Returns Ok(Some(Token)) if an indent/dedent should be emitted, Ok(None) otherwise.
    fn emit_indentation_token_if_any(
        &mut self,
        ch: char,
    ) -> Result<Option<Token>, crate::error::YamlError> {
        if ch == CHAR_SPACE || ch == CHAR_TAB {
            // Special case: leading tab(s) at line start before a flow collection.
            // YAML allows tabs as horizontal whitespace inside flow collections.
            // If the next non-whitespace character starts a flow collection, suppress
            // emitting an Indent token and treat the leading tabs/spaces as whitespace.
            if ch == CHAR_TAB && !self.in_flow {
                // Peek ahead to next non-space/tab character
                let state = self.source.save_state();
                // Consume spaces/tabs temporarily
                while let Some(c) = self.source.current() {
                    if c == CHAR_SPACE || c == CHAR_TAB {
                        self.source.next();
                    } else {
                        break;
                    }
                }
                let next_non_ws = self.source.current();
                // Restore to original position
                self.source.restore_state(state);
                if matches!(next_non_ws, Some(CHAR_LBRACKET) | Some(CHAR_LBRACE)) {
                    // Consume indentation as flow whitespace and do not emit Indent
                    let _ = self.scan_indentation(true)?; // allow tabs
                    self.at_line_start = false;
                    self.last_indent = 0;
                    return Ok(None);
                }
            }
            // In flow context, tabs that appear immediately after a newline
            // (i.e., as indentation inside a flow collection) are generally
            // forbidden by our validation rules, except when they either:
            //   - only indent a closing ']' or '}' (as in 6CA3), or
            //   - occur on a line that contains no non-whitespace content
            //     before the newline (as in Y79Y/002, which has a tab-only
            //     line between '[' and 'foo'). To support this, we peek
            //     ahead past any horizontal whitespace: if the next
            //     non-whitespace character is a flow closer, a newline/CR,
            //     or EOF (no content), we allow the tabs; otherwise, we
            //     reject them as illegal indentation in flow collections.
            if ch == CHAR_TAB && self.in_flow && self.last_was_linebreak {
                let state = self.source.save_state();
                // Consume spaces/tabs temporarily to inspect the next
                // non-whitespace character on this line.
                while let Some(c) = self.source.current() {
                    if c == CHAR_SPACE || c == CHAR_TAB {
                        self.source.next();
                    } else {
                        break;
                    }
                }
                let next_non_ws = self.source.current();
                self.source.restore_state(state);

                if !matches!(
                    next_non_ws,
                    Some(CHAR_RBRACKET)
                        | Some(CHAR_RBRACE)
                        | Some(CHAR_NEWLINE)
                        | Some(CHAR_CARRIAGE_RETURN)
                        | None
                ) {
                    return Err(crate::parser::errors::indentation_errors::IndentationErrors::tabs_not_allowed_flow_collections(
                        self.source,
                    ));
                }
            }

            let indent = self.scan_indentation(self.in_flow)?;
            self.at_line_start = false;
            self.last_indent = indent;
            // After emitting an Indent, the next token will be the first content token on this line
            self.awaiting_line_content_start = true;
            lexer_debug!("Emitting Token::Indent({})", indent);
            return Ok(Some(Token::Indent(indent)));
        } else if self.last_indent > 0 {
            // No leading space/tab, but previous indent was > 0: emit Indent(0) to signal dedent
            self.at_line_start = false;
            self.last_indent = 0;
            lexer_debug!("Emitting Token::Indent(0) (dedent to top level)");
            return Ok(Some(Token::Indent(0)));
        }
        Ok(None)
    }

    /// Scan indentation at line start
    fn scan_indentation(&mut self, in_flow: bool) -> Result<usize, crate::error::YamlError> {
        let mut count = 0;
        let mut tab_at_start = false;
        let mut saw_tab = false;
        let first_char_is_tab = self.source.current() == Some(CHAR_TAB);

        while let Some(ch) = self.source.current() {
            if ch == CHAR_SPACE {
                count += 1;
                self.source.next();
            } else if ch == CHAR_TAB {
                // Special-case: a leading tab at column 0 in block context that directly precedes a flow collection delimiter
                if !in_flow && count == 0 && first_char_is_tab {
                    if matches!(
                        self.peek_next_non_whitespace(),
                        Some(CHAR_LBRACKET)
                            | Some(CHAR_RBRACKET)
                            | Some(CHAR_LBRACE)
                            | Some(CHAR_RBRACE)
                    ) {
                        self.source.next();
                        continue;
                    }
                }
                if in_flow {
                    self.source.next();
                    continue;
                }
                if count == 0 && first_char_is_tab {
                    tab_at_start = true;
                }
                if tab_at_start {
                    if !matches!(
                        self.peek_next_non_whitespace(),
                        Some(CHAR_NEWLINE) | Some(CHAR_CARRIAGE_RETURN) | None
                    ) {
                        return Err(crate::parser::errors::indentation_errors::IndentationErrors::tabs_not_allowed_yaml_syntax(
                            self.source,
                        ));
                    }
                }
                count += 1;
                saw_tab = true;
                self.source.next();
            } else {
                break;
            }
        }
        self.last_was_linebreak = false;
        // Only mark as "space-then-tab" when the tab appeared AFTER at least one
        // space (first_char_is_tab is false). A tab-only line at column 0
        // (first_char_is_tab && saw_tab) must NOT be excluded from blank-line
        // over-indentation counting — Y79Y/000 requires it to be treated as a
        // regular blank line whose indent level triggers the error check.
        self.last_indent_had_tab = saw_tab && !first_char_is_tab;
        Ok(count)
    }

    /// Handle newline characters (LF and CR[LF]) consistently.
    /// If in flow context, suppress newline tokens and continue scanning.
    fn handle_newline(&mut self, is_cr: bool) -> Result<Option<Token>, crate::error::YamlError> {
        if is_cr {
            lexer_debug!("Emitting Token::Newline (CR) (in_flow={})", self.in_flow);
            self.source.next();
            if self.source.current() == Some(CHAR_NEWLINE) {
                self.source.next();
            }
        } else {
            lexer_debug!("Emitting Token::Newline (in_flow={})", self.in_flow);
            self.source.next();
        }

        self.at_line_start = true;
        self.last_was_linebreak = true;
        self.source_line = self.source_line.saturating_add(1);
        if self.in_flow {
            // Suppress Token::Newline in flow context
            return self.scan_token();
        }
        Ok(Some(Token::Newline))
    }
}
