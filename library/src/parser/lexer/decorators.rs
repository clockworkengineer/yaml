/// Decorator tokenization — tags (`!`/`!!`), anchors (`&`), aliases (`*`),
/// and directives (`%`).
///
/// Included into the `lexer` module from `mod.rs` via `include!("decorators.rs")`.
impl<'a> Lexer<'a> {
    /// Scan a tag: !tag or !!tag
    fn scan_tag(&mut self) -> Result<Token, crate::error::YamlError> {
        // Special case: allow a single '!' as a valid tag (YAML 1.2 plain tag)
        let peek = self.source.save_state();
        self.source.next(); // consume '!'
        if let Some(ch) = self.source.current() {
            if is_tag_delimiter(self.source, ch) {
                // It's just '!' (e.g., '! a')
                return Ok(Token::Tag("!".to_string()));
            }
        } else {
            // End of input after '!'
            return Ok(Token::Tag("!".to_string()));
        }
        self.source.restore_state(peek);
        token_scan::scan_token_with_leading(
            self.source,
            |src| {
                src.next();
            }, // consume '!'
            |src, ch| is_tag_delimiter(src, ch),
            |name| {
                // Determine if it's a double bang (!!) or single (!)
                let is_double = name.starts_with('!');
                if is_double {
                    Token::Tag(format!("!!{}", &name[1..]))
                } else {
                    Token::Tag(format!("!{}", name))
                }
            },
            false, // do not allow trailing colon for tags
            "Empty tag name",
        )
    }

    /// Scan an anchor: &name
    fn scan_anchor(&mut self) -> Result<Token, crate::error::YamlError> {
        token_scan::scan_token_with_leading(
            self.source,
            |src| {
                src.next();
            }, // consume '&'
            |src, ch| {
                // Per YAML 1.2 spec, anchor names (ns-anchor-name) exclude
                // c-indicator characters, which includes ':'.  Stop at ':'
                // when followed by a safe character or EOF so the colon
                // remains in the stream as a separate Token::Colon for the
                // mapping parser to consume.  This ensures `&anchor: value`
                // correctly produces an anchored empty key rather than
                // treating 'value' as the key with a null value.
                if ch == ':' {
                    let state = src.save_state();
                    src.next();
                    let next = src.current();
                    src.restore_state(state);
                    return matches!(
                        next,
                        None | Some(' ') | Some('\t') | Some('\n') | Some('\r')
                    );
                }
                ch.is_whitespace()
                    || ch == CHAR_NEWLINE
                    || ch == CHAR_HASH
                    || ch == CHAR_COMMA
                    || ch == CHAR_LBRACKET
                    || ch == CHAR_RBRACKET
                    || ch == CHAR_LBRACE
                    || ch == CHAR_RBRACE
            },
            Token::Anchor,
            false, // colon is now a conditional delimiter; no trailing-colon stripping needed
            "Empty anchor name",
        )
    }

    /// Scan an alias: *name
    fn scan_alias(&mut self) -> Result<Token, crate::error::YamlError> {
        token_scan::scan_token_with_leading(
            self.source,
            |src| {
                src.next();
            }, // consume '*'
            |_, ch| {
                ch.is_whitespace()
                    || ch == CHAR_NEWLINE
                    || ch == CHAR_HASH
                    || ch == CHAR_COMMA
                    || ch == CHAR_LBRACKET
                    || ch == CHAR_RBRACKET
                    || ch == CHAR_LBRACE
                    || ch == CHAR_RBRACE
            },
            Token::Alias,
            false, // do not allow trailing colon
            "Empty alias name",
        )
    }

    /// Scan a directive: %YAML or %TAG
    #[allow(dead_code)]
    fn scan_directive(&mut self) -> Result<Token, crate::error::YamlError> {
        self.source.next(); // consume '%'

        let content = self.scan_until_newline();
        Ok(Token::Directive(content.trim().to_string()))
    }
}
