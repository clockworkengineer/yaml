/// Quoted string tokenization — single-quoted and double-quoted scalars.
///
/// Included into the `lexer` module from `mod.rs` via `include!("strings.rs")`.
impl<'a> Lexer<'a> {
    /// Scan a single-quoted scalar
    fn scan_single_quoted(&mut self) -> Result<Token, crate::error::YamlError> {
        self.source.next(); // consume opening quote

        let mut content = String::new();

        loop {
            match self.source.current() {
                Some(CHAR_SINGLE_QUOTE) => {
                    self.source.next();
                    if self.source.current() == Some(CHAR_SINGLE_QUOTE) {
                        // Escaped quote
                        content.push(CHAR_SINGLE_QUOTE);
                        self.source.next();
                    } else {
                        // End of quoted string
                        // SU5Z: A comment character '#' immediately after a
                        // closing quoted scalar without any separating
                        // whitespace is invalid. Detect this here so that we
                        // can report a clear syntax error instead of
                        // silently treating it as a valid comment.
                        if let Some(CHAR_HASH) = self.source.current() {
                            return Err(crate::parser::errors::comment_errors::CommentErrors::comment_must_be_separated_from_quoted_scalar_by_whitespace(
                                self.source,
                            ));
                        }
                        break;
                    }
                }
                Some(CHAR_NEWLINE) | Some(CHAR_CARRIAGE_RETURN) => {
                    let ch = self.source.current().unwrap();
                    content.push(CHAR_NEWLINE);
                    self.source.next();
                    if ch == CHAR_CARRIAGE_RETURN && self.source.current() == Some(CHAR_NEWLINE) {
                        self.source.next();
                    }
                    // RXY3: A document-start (---) or document-end (...) marker at the
                    // beginning of a line inside a single-quoted scalar terminates the
                    // document context, making the scalar unterminated. YAML spec §7.3.3
                    // and §9.2: stream directives intercept these markers even in flow
                    // scalars. Same rule as for double-quoted scalars (9MQT).
                    let c0 = self.source.current();
                    if c0 == Some('-') || c0 == Some('.') {
                        let marker = c0.unwrap();
                        if self.peek_ahead(1) == Some(marker) && self.peek_ahead(2) == Some(marker)
                        {
                            let c3 = self.peek_ahead(3);
                            let is_doc_marker = c3.map_or(true, |c| {
                                c == CHAR_SPACE
                                    || c == CHAR_TAB
                                    || c == CHAR_NEWLINE
                                    || c == CHAR_CARRIAGE_RETURN
                            });
                            if is_doc_marker {
                                return Err(
                                    crate::parser::errors::token_errors::document_marker_in_single_quoted(
                                        self.source,
                                    ),
                                );
                            }
                        }
                    }
                }
                Some(ch) => {
                    content.push(ch);
                    self.source.next();
                }
                None => {
                    lexer_debug!("Unterminated single-quoted string: reached EOF");
                    return Err(
                        crate::parser::errors::token_errors::unterminated_single_quoted_eof(
                            self.source,
                        ),
                    );
                }
            }
        }

        Ok(Token::SingleQuoted(content))
    }

    /// Scan a double-quoted scalar
    fn scan_double_quoted(&mut self) -> Result<Token, crate::error::YamlError> {
        self.source.next(); // consume opening quote

        let mut content = String::new();
        // Track whether the quoted string literally crosses a newline in the source
        // (as opposed to an escape sequence like \n). Multiline double-quoted strings
        // cannot be used as implicit block mapping keys (YAML spec §8.1.1 / 7LBH).
        let mut crossed_newline = false;
        // Minimum count of leading SPACE characters on any continuation line
        // (tabs never count as spaces per §6.1). usize::MAX = no line crossing.
        let mut min_cont_spaces: usize = usize::MAX;

        loop {
            match self.source.current() {
                Some('\\') => {
                    self.source.next();
                    match self.source.current() {
                        // Line folding: backslash followed by a line break (CR, LF, or CRLF)
                        // YAML allows escaping a line break in double-quoted scalars; the newline
                        // and any following indentation are suppressed (no characters added).
                        Some(c) if c == CHAR_NEWLINE || c == CHAR_CARRIAGE_RETURN => {
                            // Consume CR and optional LF
                            if c == CHAR_CARRIAGE_RETURN {
                                self.source.next();
                                if self.source.current() == Some(CHAR_NEWLINE) {
                                    self.source.next();
                                }
                            } else {
                                // LF
                                self.source.next();
                            }
                            // Suppress any following indentation (spaces or tabs)
                            while let Some(ws) = self.source.current() {
                                if ws == CHAR_SPACE || ws == CHAR_TAB {
                                    self.source.next();
                                } else {
                                    break;
                                }
                            }
                            // Do not push any character; continue scanning content
                            continue;
                        }
                        Some('0') => {
                            content.push('\0');
                            self.source.next();
                        }
                        Some('a') => {
                            content.push('\x07');
                            self.source.next();
                        }
                        Some('b') => {
                            content.push('\x08');
                            self.source.next();
                        }
                        Some('t') | Some('\t') => {
                            content.push('\t');
                            self.source.next();
                        }
                        Some('n') => {
                            content.push('\n');
                            self.source.next();
                        }
                        Some('v') => {
                            content.push('\x0B');
                            self.source.next();
                        }
                        Some('f') => {
                            content.push('\x0C');
                            self.source.next();
                        }
                        Some('r') => {
                            content.push('\r');
                            self.source.next();
                        }
                        Some('e') => {
                            content.push('\x1B');
                            self.source.next();
                        }
                        Some(' ') => {
                            content.push(' ');
                            self.source.next();
                        }
                        Some('"') => {
                            content.push('"');
                            self.source.next();
                        }
                        Some('/') => {
                            content.push('/');
                            self.source.next();
                        }
                        Some('\\') => {
                            content.push('\\');
                            self.source.next();
                        }
                        Some('N') => {
                            content.push('\u{0085}');
                            self.source.next();
                        }
                        Some('_') => {
                            content.push('\u{00A0}');
                            self.source.next();
                        }
                        Some('L') => {
                            content.push('\u{2028}');
                            self.source.next();
                        }
                        Some('P') => {
                            content.push('\u{2029}');
                            self.source.next();
                        }
                        Some('x') => {
                            // \xXX - 2 hex digits
                            self.source.next();
                            let mut hex = String::new();
                            for _ in 0..2 {
                                match self.source.current() {
                                    Some(c) if c.is_ascii_hexdigit() => {
                                        hex.push(c);
                                        self.source.next();
                                    }
                                    _ => {
                                        return Err(
                                            crate::parser::errors::token_errors::invalid_escape_x_expected_2_hex(
                                                self.source,
                                            ),
                                        );
                                    }
                                }
                            }
                            let code = u8::from_str_radix(&hex, 16).unwrap();
                            content.push(code as char);
                        }
                        Some('u') => {
                            // \uXXXX - 4 hex digits
                            self.source.next();
                            let mut hex = String::new();
                            for _ in 0..4 {
                                match self.source.current() {
                                    Some(c) if c.is_ascii_hexdigit() => {
                                        hex.push(c);
                                        self.source.next();
                                    }
                                    _ => {
                                        return Err(
                                            crate::parser::errors::token_errors::invalid_escape_u_expected_4_hex(
                                                self.source,
                                            ),
                                        );
                                    }
                                }
                            }
                            let code = u32::from_str_radix(&hex, 16).unwrap();
                            match char::from_u32(code) {
                                Some(ch) => content.push(ch),
                                None => {
                                    return Err(
                                        crate::parser::errors::token_errors::invalid_unicode_codepoint_u4(
                                            self.source,
                                            code,
                                        ),
                                    );
                                }
                            }
                        }
                        Some('U') => {
                            // \UXXXXXXXX - 8 hex digits
                            self.source.next();
                            let mut hex = String::new();
                            for _ in 0..8 {
                                match self.source.current() {
                                    Some(c) if c.is_ascii_hexdigit() => {
                                        hex.push(c);
                                        self.source.next();
                                    }
                                    _ => {
                                        return Err(
                                            crate::parser::errors::token_errors::invalid_escape_u_expected_8_hex(
                                                self.source,
                                            ),
                                        );
                                    }
                                }
                            }
                            let code = u32::from_str_radix(&hex, 16).unwrap();
                            match char::from_u32(code) {
                                Some(ch) => content.push(ch),
                                None => {
                                    return Err(
                                        crate::parser::errors::token_errors::invalid_unicode_codepoint_u8(
                                            self.source,
                                            code,
                                        ),
                                    );
                                }
                            }
                        }
                        // Invalid escape sequences - reject per YAML 1.2 spec
                        Some(c) => {
                            return Err(
                                crate::parser::errors::token_errors::invalid_escape_generic(
                                    self.source,
                                    c,
                                ),
                            );
                        }
                        None => {
                            lexer_debug!(
                                "Unterminated double-quoted string: reached EOF after escape"
                            );
                            return Err(
                                crate::parser::errors::token_errors::unterminated_double_quoted_eof_after_escape(
                                    self.source,
                                ),
                            );
                        }
                    }
                }
                Some(CHAR_DOUBLE_QUOTE) => {
                    self.source.next();
                    // SU5Z: A comment character '#' immediately after a
                    // closing quoted scalar without any separating
                    // whitespace is invalid. Detect this here so that we
                    // can report a clear syntax error instead of
                    // silently treating it as a valid comment.
                    if let Some(CHAR_HASH) = self.source.current() {
                        return Err(crate::parser::errors::comment_errors::CommentErrors::comment_must_be_separated_from_quoted_scalar_by_whitespace(
                            self.source,
                        ));
                    }
                    break;
                }
                Some(CHAR_NEWLINE) | Some(CHAR_CARRIAGE_RETURN) => {
                    let ch = self.source.current().unwrap();
                    // Normalise CR / CRLF → LF in content
                    content.push(CHAR_NEWLINE);
                    // Mark that this double-quoted string spans multiple source lines.
                    crossed_newline = true;
                    self.source.next();
                    if ch == CHAR_CARRIAGE_RETURN && self.source.current() == Some(CHAR_NEWLINE) {
                        self.source.next();
                    }
                    // YAML 1.2: a document-start (---) or document-end (...) marker
                    // at the beginning of a new line inside a flow scalar terminates
                    // the document context, making the double-quoted string unterminated.
                    let c0 = self.source.current();
                    if c0 == Some('-') || c0 == Some('.') {
                        let marker = c0.unwrap();
                        if self.peek_ahead(1) == Some(marker) && self.peek_ahead(2) == Some(marker)
                        {
                            let c3 = self.peek_ahead(3);
                            let is_doc_marker = c3.map_or(true, |c| {
                                c == CHAR_SPACE
                                    || c == CHAR_TAB
                                    || c == CHAR_NEWLINE
                                    || c == CHAR_CARRIAGE_RETURN
                            });
                            if is_doc_marker {
                                return Err(
                                    crate::parser::errors::token_errors::document_marker_in_double_quoted(
                                        self.source,
                                    ),
                                );
                            }
                        }
                    }
                    // Count leading SPACE characters on the continuation line
                    // (tabs do NOT count as spaces for indentation purposes per YAML §6.1).
                    // Track the minimum across all continuation lines so the parser can
                    // validate that the continuation is more indented than the enclosing
                    // block context.  Skip empty lines (next char is \r, \n, or EOF);
                    // only non-empty continuation lines count for the indentation check.
                    {
                        let mut sp = 0usize;
                        let mut peek_pos = 0;
                        loop {
                            match self.peek_ahead(peek_pos) {
                                Some(CHAR_SPACE) => {
                                    sp += 1;
                                    peek_pos += 1;
                                }
                                _ => break,
                            }
                        }
                        let next_after = self.peek_ahead(peek_pos);
                        let is_empty_line = matches!(
                            next_after,
                            None | Some(CHAR_NEWLINE) | Some(CHAR_CARRIAGE_RETURN)
                        );
                        if !is_empty_line && sp < min_cont_spaces {
                            min_cont_spaces = sp;
                        }
                    }
                }
                Some(ch) => {
                    content.push(ch);
                    self.source.next();
                }
                None => {
                    lexer_debug!("Unterminated double-quoted string: reached EOF");
                    return Err(
                        crate::parser::errors::token_errors::unterminated_double_quoted_eof(
                            self.source,
                        ),
                    );
                }
            }
        }

        Ok(Token::DoubleQuoted(
            content,
            crossed_newline,
            min_cont_spaces,
        ))
    }
}
