/// Flow collection token validation.
///
/// Included into the `lexer` module from `mod.rs` via `include!("flow.rs")`.
impl<'a> Lexer<'a> {
    /// Validate what may follow immediately after a flow closer ('}' or ']')
    /// Consumes any horizontal whitespace; enforces that a comment must be preceded by whitespace.
    /// Also validates that the next non-whitespace character is an allowed delimiter or line break.
    ///
    /// YAML 1.2 requires that after a flow closer, the next significant character
    /// be one of: newline, ',', another flow closer ('}', ']'), '#', or ':' (for
    /// cases like "[a]: 1" where the flow collection is a mapping key). Any other
    /// immediate content on the same line is invalid, even if separated by spaces.
    fn validate_post_flow_closer(&mut self, closer: char) -> Result<(), crate::error::YamlError> {
        // Allow horizontal whitespace
        let mut saw_whitespace = false;
        while let Some(c) = self.source.current() {
            if c == CHAR_SPACE || c == CHAR_TAB {
                saw_whitespace = true;
                self.source.next();
            } else {
                break;
            }
        }
        // If a comment follows, it must be preceded by whitespace
        if let Some(CHAR_HASH) = self.source.current() {
            if !saw_whitespace {
                return Err(crate::parser::errors::comment_errors::CommentErrors::comment_must_be_preceded_by_whitespace_after_flow_closer(
                    self.source,
                    closer,
                ));
            }
        }
        // Validate next non-whitespace character regardless of whether whitespace was seen
        if let Some(c) = self.source.current() {
            let is_allowed = c == CHAR_NEWLINE
                || c == CHAR_CARRIAGE_RETURN
                || c == CHAR_COMMA
                || c == CHAR_RBRACKET
                || c == CHAR_RBRACE
                || c == CHAR_HASH
                || c == CHAR_COLON;
            if !is_allowed {
                return Err(crate::parser::document::flow_punctuation::invalid_content_immediately_after_flow_closer(
                    self.source,
                    closer,
                    c,
                ));
            }
        }
        Ok(())
    }
}
