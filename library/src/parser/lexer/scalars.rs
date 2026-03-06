/// Plain (unquoted) scalar tokenization.
///
/// Included into the `lexer` module from `mod.rs` via `include!("scalars.rs")`.
impl<'a> Lexer<'a> {
    /// Scan a plain (unquoted) scalar
    fn scan_plain_scalar(&mut self) -> Result<Token, crate::error::YamlError> {
        let mut content = String::new();

        loop {
            match self.source.current() {
                // In flow context, allow line breaks inside a plain scalar
                // and fold them (plus any following spaces) into a single
                // space. This matches YAML's treatment of plain scalars in
                // flow collections (e.g., the UT92 test case where a key is
                // split across lines: "{ matches\n% : 20 }"). Tabs remain a
                // terminator and are not treated as indentation here.
                Some(ch) if self.in_flow && (ch == CHAR_NEWLINE || ch == CHAR_CARRIAGE_RETURN) => {
                    // Consume the newline or CRLF
                    if ch == CHAR_CARRIAGE_RETURN {
                        self.source.next();
                        if self.source.current() == Some(CHAR_NEWLINE) {
                            self.source.next();
                        }
                    } else {
                        self.source.next();
                    }
                    // Track the newline for multiline-key detection even
                    // though we do not emit a Token::Newline in flow context.
                    self.source_line = self.source_line.saturating_add(1);
                    // Consume any following spaces (but not tabs) so we
                    // don't accumulate multiple spaces.
                    while let Some(ws) = self.source.current() {
                        if ws == CHAR_SPACE {
                            self.source.next();
                        } else {
                            break;
                        }
                    }
                    if !content.is_empty() {
                        content.push(CHAR_SPACE);
                    }
                }
                Some(ch) if ch == CHAR_COLON => {
                    // Colon could be part of scalar or a separator
                    let state = self.source.save_state();
                    self.source.next();

                    match self.source.current() {
                        Some(c) if c.is_whitespace() || c == CHAR_NEWLINE => {
                            // Colon followed by whitespace - it's a separator
                            self.source.restore_state(state);
                            break;
                        }
                        None => {
                            // Colon at EOF - it's a separator
                            self.source.restore_state(state);
                            break;
                        }
                        _ => {
                            // Colon followed by content - part of scalar
                            self.source.restore_state(state);
                            content.push(ch);
                            self.source.next();
                        }
                    }
                }
                Some(ch) if ch == CHAR_HASH && !content.is_empty() => {
                    // Hash could be a comment start
                    let prev = content.chars().last();
                    if prev.map_or(false, |c| c.is_whitespace()) {
                        // Hash after whitespace - it's a comment
                        break;
                    }
                    content.push(ch);
                    self.source.next();
                }
                Some(ch) if ch.is_whitespace() && ch != CHAR_SPACE => {
                    // Newline, tab, etc. - end of scalar in block context
                    // or when not handled by the flow-specific branch above.
                    break;
                }
                Some(ch)
                    if self.in_flow
                        && (ch == CHAR_COMMA
                            || ch == CHAR_LBRACKET
                            || ch == CHAR_RBRACKET
                            || ch == CHAR_LBRACE
                            || ch == CHAR_RBRACE) =>
                {
                    // In flow context, these characters act as flow
                    // indicators and terminate the plain scalar. Outside
                    // of flow, they are allowed as part of the scalar
                    // content (e.g., the ']' in "bla]keks" in AZW3).
                    break;
                }
                Some(ch) => {
                    content.push(ch);
                    self.source.next();
                }
                None => break,
            }
        }

        Ok(Token::Plain(content.trim_end().to_string()))
    }
}
