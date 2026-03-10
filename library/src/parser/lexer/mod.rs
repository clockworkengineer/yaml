use crate::lexer_debug;

/**
 * YAML Lexer / Tokenizer
 *
 * Implements the tokenization layer for YAML parsing, sitting between the character source
 * and the parser. Pre-processes decorators (tags, anchors), identifies token boundaries,
 * and simplifies the parser by avoiding infinite lookahead loops.
 *
 * This module is split into focused sub-files included at the bottom:
 *   block.rs      — indentation, dedent, and newline handling
 *   flow.rs       — flow-closer validation
 *   decorators.rs — tag, anchor, alias, and directive scanning
 *   strings.rs    — single-quoted and double-quoted scalar scanning
 *   scalars.rs    — plain scalar scanning
 *
 * Copyright (c) 2026 YAML Library Developers
 */
use crate::constants::*;
use crate::io::traits::ISource;
use crate::parser::utils::error_helpers;
use crate::parser::utils::token_scan;

// Helper for tag delimiter logic to avoid borrow checker issues
#[inline]
fn is_tag_delimiter(source: &mut dyn ISource, ch: char) -> bool {
    if ch == CHAR_COLON {
        let state = source.save_state();
        source.next();
        let next_ch = source.current();
        source.restore_state(state);
        if next_ch.map_or(true, |c| c.is_whitespace() || c == CHAR_NEWLINE) {
            return true;
        }
    }
    ch.is_whitespace()
        || ch == CHAR_NEWLINE
        || ch == CHAR_HASH
        || ch == CHAR_COMMA
        || ch == CHAR_LBRACKET
        || ch == CHAR_RBRACKET
        || ch == CHAR_LBRACE
        || ch == CHAR_RBRACE
}

#[cfg(feature = "debug-trace")]
#[inline]
pub fn lexer_log(msg: String) {
    #[cfg(feature = "std")]
    {
        if let Ok(v) = std::env::var("YAML_TRACE_LEXER") {
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

/// A YAML token with its type and associated data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Tag decorator: !tag or !!tag
    Tag(String),

    /// Anchor definition: &name
    Anchor(String),

    /// Alias reference: *name
    Alias(String),

    /// Start of flow mapping: {
    FlowMappingStart,

    /// End of flow mapping: }
    FlowMappingEnd,

    /// Start of flow sequence: [
    FlowSequenceStart,

    /// End of flow sequence: ]
    FlowSequenceEnd,

    /// Comma separator in flow collections
    Comma,

    /// Colon key-value separator
    Colon,

    /// Dash sequence indicator: -
    Dash,

    /// Question mark explicit key indicator: ?
    QuestionMark,

    /// Single-quoted scalar content
    SingleQuoted(String),

    /// Double-quoted scalar content
    DoubleQuoted(String, bool, usize), // bool = true if spanned multiple lines; usize = min leading spaces on any continuation line (usize::MAX = no continuation)

    /// Plain scalar content (unquoted)
    Plain(String),

    /// Newline
    Newline,

    /// Indentation (spaces at line start)
    Indent(usize),

    /// Comment content (everything after #)
    Comment(String),

    /// Document start marker: ---
    DocumentStart,

    /// Document end marker: ...
    DocumentEnd,

    /// Directive: %YAML or %TAG
    #[allow(dead_code)]
    Directive(String),

    /// End of stream
    Eof,
}

/// Tokenizer state for YAML lexical analysis
pub struct Lexer<'a> {
    pub(crate) source: &'a mut dyn ISource,
    current_token: Option<Token>,
    peeked_token: Option<Token>,
    at_line_start: bool,
    in_flow: bool, // Track if we're in flow context
    // Track whether the last processed character was a line break
    // Used to distinguish indentation after newline vs. leading tabs at stream start in flow
    last_was_linebreak: bool,
    last_indent: usize, // Track last emitted indent
    // True when the next non-indentation token will be the first content token on the line
    awaiting_line_content_start: bool,
    // True if the most recently emitted non-indentation token started the line's content
    last_token_started_line: bool,
    // True if the most recently scanned indentation included a tab character.
    // Used by parse_block_scalar to distinguish pure-space blank lines from
    // tab-containing lines (which are actually more-indented content lines).
    last_indent_had_tab: bool,
    // Cumulative count of newlines seen in the source (0-based, flow-transparent).
    // Incremented in handle_newline regardless of flow context so that callers
    // can detect multi-line keys even when Newline tokens are suppressed.
    source_line: usize,
    // The value of source_line at the START of the scan that produced
    // current_token.  Set at the beginning of every scan_token() call.
    // Because scan_plain_scalar in flow context folds newlines internally
    // (incrementing source_line but NOT calling handle_newline recursively),
    // this field captures where the scan BEGAN, letting callers detect that
    // a plain scalar's scan crossed a line even though the caller only sees
    // the completed token.  Compare against source_line() after the token
    // is consumed to determine whether a line boundary was crossed.
    token_start_source_line: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer wrapping a character source
    pub fn new(source: &'a mut dyn ISource, in_flow: bool) -> Self {
        Lexer {
            source,
            current_token: None,
            peeked_token: None,
            at_line_start: true,
            in_flow,
            last_was_linebreak: false,
            last_indent: 0,
            awaiting_line_content_start: false,
            last_token_started_line: false,
            last_indent_had_tab: false,
            source_line: 0,
            token_start_source_line: 0,
        }
    }

    /// Set whether the lexer is currently inside a flow collection context.
    ///
    /// When `in_flow` is true, newlines are suppressed as tokens and tabs are
    /// treated as allowable horizontal whitespace instead of invalid indentation.
    pub fn set_in_flow(&mut self, in_flow: bool) {
        self.in_flow = in_flow;
    }

    /// Returns true if the last processed character was a line break and no
    /// indentation token was emitted before the current token.  This means the
    /// current token sits at column 0 on a new line (inside a flow context,
    /// where newlines are suppressed rather than emitted as tokens).
    ///
    /// Used by the parser to detect flow collection content at the outer-block
    /// indent level (required by the YAML spec, e.g. VJP3/00).
    #[inline]
    pub(crate) fn last_was_linebreak(&self) -> bool {
        self.last_was_linebreak
    }

    /// Returns true if the most recently scanned indentation included at least
    /// one tab character.  Lines whose indentation contained a tab are treated
    /// as more-indented content lines in block scalars (the tab becomes part
    /// of the scalar's content) rather than pure blank lines.
    ///
    /// IMPORTANT: Only returns true when spaces PRECEDE the tab (space+tab).
    /// A tab-only line at column 0 does NOT set this flag — it should still be
    /// counted as a blank line for the blank-indent over-indentation check.
    /// This distinguishes Y79Y/000 (tab-only → error) from Y79Y/001 and R4YG
    /// (space+tab → valid content line with tab as the scalar payload).
    #[inline]
    pub(crate) fn last_indent_had_tab(&self) -> bool {
        self.last_indent_had_tab
    }

    /// Returns the number of newlines seen in the source so far (0-based).
    /// This counter is incremented for every line break encountered, including
    /// those inside flow collections where `Token::Newline` is suppressed.
    /// Callers can snapshot this value before parsing a key and compare after
    /// to detect multi-line keys in all contexts.
    #[inline]
    pub(crate) fn source_line(&self) -> usize {
        self.source_line
    }

    /// Returns the source-line index at the START of the scan that produced
    /// `current_token`.  Because scan_plain_scalar in flow context folds
    /// newlines internally (advancing source_line but not emitting a
    /// Token::Newline), `source_line()` is already incremented by the time
    /// the token is visible.  Comparing `source_line()` against this value
    /// after consuming the token reveals whether the scan crossed a line.
    #[inline]
    pub(crate) fn token_start_source_line(&self) -> usize {
        self.token_start_source_line
    }

    /// Get the current token without consuming it
    #[inline]
    pub fn current(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    /// Advance to the next token
    #[inline]
    pub fn next(&mut self) -> Result<Option<Token>, crate::error::YamlError> {
        if let Some(peeked) = self.peeked_token.take() {
            self.current_token = Some(peeked.clone());
            return Ok(Some(peeked));
        }

        let token = self.scan_token()?;
        self.current_token = token.clone();
        Ok(token)
    }

    /// Peek at the next token without consuming it
    #[inline]
    pub fn peek(&mut self) -> Result<Option<&Token>, crate::error::YamlError> {
        if self.peeked_token.is_none() {
            self.peeked_token = self.scan_token()?;
        }
        Ok(self.peeked_token.as_ref())
    }

    /// Scan and return the next token from the source
    fn scan_token(&mut self) -> Result<Option<Token>, crate::error::YamlError> {
        // Record where this scan started (used for multiline-key detection:
        // after the token is consumed, comparing source_line() > token_start_source_line()
        // reveals whether the scan crossed a line boundary, even in flow context
        // where Token::Newline is suppressed and the fold happens inside
        // scan_plain_scalar rather than handle_newline).
        self.token_start_source_line = self.source_line;
        // If we're awaiting the first content token of a line but a non-indentation
        // token has already been emitted, clear the flag to avoid misclassifying
        // mid-line tokens as starting the line's content.
        if self.awaiting_line_content_start {
            if let Some(prev) = self.current_token.as_ref() {
                if !matches!(prev, Token::Indent(_) | Token::Newline) {
                    self.awaiting_line_content_start = false;
                }
            }
        }
        // Skip whitespace (except at line start where it's indentation)
        if !self.at_line_start {
            // YAML forbids using tabs as indentation in block context, including
            // the horizontal whitespace immediately following block indicators
            // like '-', '?', and ':' at the start of a content span. If the last
            // emitted token was one of these indicators and we're not in a flow
            // collection, validate that the upcoming horizontal whitespace does
            // not contain any tabs before the next non-whitespace character.
            if !self.in_flow {
                if let Some(prev_tok) = self.current_token.clone() {
                    match prev_tok {
                        Token::Dash => {
                            // After a sequence '-' at line start, tabs are only forbidden
                            // when they serve as indentation before a nested indicator (e.g., '-\t-').
                            let state = self.source.save_state();
                            let mut saw_tab = false;
                            while let Some(c) = self.source.current() {
                                if c == CHAR_SPACE || c == CHAR_TAB {
                                    if c == CHAR_TAB {
                                        saw_tab = true;
                                    }
                                    self.source.next();
                                } else {
                                    break;
                                }
                            }
                            let c1 = self.source.current();
                            // Peek one more to distinguish nested '-' from scalar starting with '-'
                            let c2 = if c1.is_some() {
                                self.peek_ahead(1)
                            } else {
                                None
                            };
                            self.source.restore_state(state);
                            let nested_indicator = match c1 {
                                Some('-') => match c2 {
                                    Some(n)
                                        if n == CHAR_SPACE
                                            || n == CHAR_TAB
                                            || n == CHAR_NEWLINE
                                            || n == CHAR_CARRIAGE_RETURN =>
                                    {
                                        true
                                    }
                                    None => true,
                                    _ => false,
                                },
                                Some('?') | Some(CHAR_COLON) => true,
                                _ => false,
                            };
                            if saw_tab && nested_indicator {
                                return Err(crate::parser::errors::indentation_errors::IndentationErrors::tabs_not_allowed_yaml_syntax(
                                    self.source,
                                ));
                            }
                        }
                        Token::QuestionMark => {
                            // After explicit key '?', tabs in separating whitespace are not allowed.
                            let state = self.source.save_state();
                            let mut saw_tab = false;
                            while let Some(c) = self.source.current() {
                                if c == CHAR_SPACE || c == CHAR_TAB {
                                    if c == CHAR_TAB {
                                        saw_tab = true;
                                    }
                                    self.source.next();
                                } else {
                                    break;
                                }
                            }
                            self.source.restore_state(state);
                            if saw_tab {
                                return Err(crate::parser::errors::indentation_errors::IndentationErrors::tabs_not_allowed_yaml_syntax(
                                    self.source,
                                ));
                            }
                        }
                        Token::Colon => {
                            // Only forbid tabs after ':' when ':' started this line's content
                            if self.last_token_started_line {
                                let state = self.source.save_state();
                                let mut saw_tab = false;
                                while let Some(c) = self.source.current() {
                                    if c == CHAR_SPACE || c == CHAR_TAB {
                                        if c == CHAR_TAB {
                                            saw_tab = true;
                                        }
                                        self.source.next();
                                    } else {
                                        break;
                                    }
                                }
                                self.source.restore_state(state);
                                if saw_tab {
                                    return Err(crate::parser::errors::indentation_errors::IndentationErrors::tabs_not_allowed_yaml_syntax(
                                        self.source,
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            self.skip_horizontal_whitespace();
        }

        let ch = match self.source.current() {
            Some(c) => c,
            None => {
                lexer_debug!("Emitting Token::Eof");
                return Ok(Some(Token::Eof));
            }
        };

        // Handle line-start indentation via helper to keep logic centralized
        if self.at_line_start {
            if let Some(tok) = self.emit_indentation_token_if_any(ch)? {
                return Ok(Some(tok));
            }
            // No indentation to emit; the next token will start the line's content.
            self.awaiting_line_content_start = true;
        }

        self.at_line_start = false;

        // Match token types
        match ch {
            CHAR_NEWLINE => self.handle_newline(false),
            CHAR_CARRIAGE_RETURN => self.handle_newline(true),
            CHAR_HASH => {
                self.source.next();
                let comment = self.scan_until_newline();
                Ok(Some(Token::Comment(comment)))
            }
            '!' => {
                let tok = self.scan_tag()?;
                let result = Some(tok.clone());
                lexer_debug!("Emitting Token::{:?}", tok);
                Ok(result)
            }
            '&' => {
                let tok = self.scan_anchor()?;
                let result = Some(tok.clone());
                lexer_debug!("Emitting Token::{:?}", tok);
                Ok(result)
            }
            CHAR_ASTERISK => {
                // Disambiguate between an alias token (e.g. "*anchor") and
                // plain scalar content starting with '*', such as lines inside
                // block scalars (e.g. "* bullet").
                //
                // We only treat '*' as an alias indicator when it is
                // immediately followed by a non-whitespace character on the
                // same line. Otherwise, we fall back to scanning a plain
                // scalar starting with '*'. This avoids emitting alias
                // tokens (and "Empty alias name" errors) for content inside
                // block scalars like the YAML test 7T8X.
                let state = self.source.save_state();
                self.source.next(); // temporarily consume '*'
                let next_ch = self.source.current();
                self.source.restore_state(state);

                // Treat '*' as an alias only when the very next
                // character is not a space or tab. This preserves
                // errors for truly empty aliases like "*\n" (where the
                // next char is a newline), while allowing constructs
                // like "* bullet" inside block scalars to be lexed as
                // plain scalars.
                let treat_as_alias = matches!(
                    next_ch,
                    Some(c) if c != CHAR_SPACE && c != '\t'
                );

                if treat_as_alias {
                    let tok = self.scan_alias()?;
                    let result = Some(tok.clone());
                    lexer_debug!("Emitting Token::{:?}", tok);
                    Ok(result)
                } else {
                    lexer_debug!("Emitting Token::Plain (leading '*')");
                    Ok(Some(self.scan_plain_scalar()?))
                }
            }
            CHAR_LBRACE => {
                self.source.next();
                self.last_was_linebreak = false;
                lexer_debug!("Emitting Token::FlowMappingStart");
                Ok(Some(Token::FlowMappingStart))
            }
            CHAR_RBRACE => {
                self.source.next();
                self.validate_post_flow_closer('}')?;
                self.last_was_linebreak = false;
                lexer_debug!("Emitting Token::FlowMappingEnd");
                Ok(Some(Token::FlowMappingEnd))
            }
            CHAR_LBRACKET => {
                self.source.next();
                self.last_was_linebreak = false;
                Ok(Some(Token::FlowSequenceStart))
            }
            CHAR_RBRACKET => {
                self.source.next();
                self.validate_post_flow_closer(']')?;
                self.last_was_linebreak = false;
                Ok(Some(Token::FlowSequenceEnd))
            }
            CHAR_COMMA => {
                self.source.next();
                // Allow any horizontal whitespace before comment
                let mut seen_whitespace = false;
                while let Some(c) = self.source.current() {
                    if c == CHAR_SPACE || c == CHAR_TAB {
                        seen_whitespace = true;
                        self.source.next();
                    } else {
                        break;
                    }
                }
                if let Some(CHAR_HASH) = self.source.current() {
                    if !seen_whitespace {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "DEBUG: Comment after comma with no whitespace at {:?}",
                            self.source.current()
                        );
                        return Err(error_helpers::syntax_error(
                            self.source,
                            "YAML syntax error: comment must be preceded by whitespace after ',' in flow collection",
                        ));
                    }
                }
                self.last_was_linebreak = false;
                Ok(Some(Token::Comma))
            }
            CHAR_COLON => {
                // Colon can be a key-value separator or part of a plain scalar.
                // In flow context, always treat ':' as a separator (no whitespace required).
                // In block context, treat ':' followed by whitespace/newline/EOF as separator;
                // otherwise include it as part of the plain scalar.
                if self.in_flow {
                    self.source.next();
                    self.last_was_linebreak = false;
                    // Track whether this ':' started the line's content
                    self.last_token_started_line = self.awaiting_line_content_start;
                    self.awaiting_line_content_start = false;
                    Ok(Some(Token::Colon))
                } else {
                    let state = self.source.save_state();
                    self.source.next();
                    let next_ch = self.source.current();
                    if next_ch.map_or(true, |c| c.is_whitespace() || c == CHAR_NEWLINE) {
                        // Separator in block context
                        self.last_was_linebreak = false;
                        // Track whether this ':' started the line's content
                        self.last_token_started_line = self.awaiting_line_content_start;
                        self.awaiting_line_content_start = false;
                        Ok(Some(Token::Colon))
                    } else {
                        // Part of scalar, restore and scan as plain scalar
                        self.source.restore_state(state);
                        self.last_was_linebreak = false;
                        Ok(Some(self.scan_plain_scalar()?))
                    }
                }
            }
            CHAR_DASH => {
                // Could be: dash (sequence), document start (---), or plain scalar
                let state = self.source.save_state();
                self.source.next();

                match (self.source.current(), self.peek_ahead(1)) {
                    (Some('-'), Some('-')) => {
                        // Document start: ---
                        // Only emit at indent 1
                        let indent = self.indent_level();
                        if indent == 1 {
                            self.source.next();
                            self.source.next();
                            Ok(Some(Token::DocumentStart))
                        } else {
                            // Not at indent 0, treat as plain scalar
                            self.source.restore_state(state);
                            self.last_was_linebreak = false;
                            Ok(Some(self.scan_plain_scalar()?))
                        }
                    }
                    (Some(c), _) if c.is_whitespace() || c == CHAR_NEWLINE => {
                        // Dash (sequence indicator)
                        self.last_was_linebreak = false;
                        Ok(Some(Token::Dash))
                    }
                    _ => {
                        // Plain scalar starting with dash
                        self.source.restore_state(state);
                        self.last_was_linebreak = false;
                        Ok(Some(self.scan_plain_scalar()?))
                    }
                }
            }
            CHAR_DOT => {
                // Could be: document end (...) or plain scalar
                let state = self.source.save_state();
                self.source.next();

                match (self.source.current(), self.peek_ahead(1)) {
                    (Some('.'), Some('.')) => {
                        // Document end: ...
                        // Only emit at indent 0
                        let indent = self.indent_level();
                        if indent == 1 {
                            self.source.next();
                            self.source.next();
                            Ok(Some(Token::DocumentEnd))
                        } else {
                            // Not at indent 0, treat as plain scalar
                            self.source.restore_state(state);
                            self.last_was_linebreak = false;
                            Ok(Some(self.scan_plain_scalar()?))
                        }
                    }
                    _ => {
                        // Plain scalar starting with dot
                        self.source.restore_state(state);
                        self.last_was_linebreak = false;
                        Ok(Some(self.scan_plain_scalar()?))
                    }
                }
            }
            CHAR_QUESTION_MARK => {
                self.source.next();
                self.last_was_linebreak = false;
                Ok(Some(Token::QuestionMark))
            }
            CHAR_PERCENT => {
                // Treat '%' as part of a plain scalar for token-based
                // parsing. Top-level YAML directives (e.g. %YAML, %TAG)
                // are handled separately by the document parser using the
                // raw character stream, so emitting a dedicated Directive
                // token here would only interfere with content such as
                // flow mappings that legitimately contain '%'.
                Ok(Some(self.scan_plain_scalar()?))
            }
            CHAR_SINGLE_QUOTE => Ok(Some(self.scan_single_quoted()?)),
            CHAR_DOUBLE_QUOTE => Ok(Some(self.scan_double_quoted()?)),
            _ => Ok(Some(self.scan_plain_scalar()?)),
        }
    }

    /// Scan until end of current line (exclusive of the newline character).
    /// Used for comment bodies and directive content.
    fn scan_until_newline(&mut self) -> String {
        let mut content = String::new();
        while let Some(ch) = self.source.current() {
            if ch == CHAR_NEWLINE || ch == CHAR_CARRIAGE_RETURN {
                break;
            }
            content.push(ch);
            self.source.next();
        }
        content
    }

    /// Peek ahead n characters without consuming them.
    fn peek_ahead(&mut self, n: usize) -> Option<char> {
        let state = self.source.save_state();
        for _ in 0..n {
            if self.source.current().is_some() {
                self.source.next();
            }
        }
        let result = self.source.current();
        self.source.restore_state(state);
        result
    }

    /// Get the current indentation level (for error reporting)
    pub fn indent_level(&self) -> usize {
        self.source.get_current_indent_level()
    }

    /// Returns the indentation level of the current line (number of leading spaces/tabs).
    pub fn line_indent(&self) -> usize {
        self.last_indent
    }

    /// Take a snapshot of the lexer's token-cache and boolean state for
    /// speculative probing (look-ahead without permanently consuming tokens).
    pub(crate) fn snapshot(&self) -> LexerSnapshot {
        LexerSnapshot {
            current_token: self.current_token.clone(),
            peeked_token: self.peeked_token.clone(),
            at_line_start: self.at_line_start,
            in_flow: self.in_flow,
            last_was_linebreak: self.last_was_linebreak,
            last_indent: self.last_indent,
            awaiting_line_content_start: self.awaiting_line_content_start,
            last_token_started_line: self.last_token_started_line,
        }
    }

    /// Restore a previously-taken lexer snapshot (used after speculative probing).
    pub(crate) fn restore_snapshot(&mut self, snap: LexerSnapshot) {
        self.current_token = snap.current_token;
        self.peeked_token = snap.peeked_token;
        self.at_line_start = snap.at_line_start;
        self.in_flow = snap.in_flow;
        self.last_was_linebreak = snap.last_was_linebreak;
        self.last_indent = snap.last_indent;
        self.awaiting_line_content_start = snap.awaiting_line_content_start;
        self.last_token_started_line = snap.last_token_started_line;
    }
}

/// Snapshot of the lexer's token-cache and boolean state.
/// Obtained via [`Lexer::snapshot`] and restored via [`Lexer::restore_snapshot`].
#[derive(Clone, Debug)]
pub(crate) struct LexerSnapshot {
    current_token: Option<Token>,
    peeked_token: Option<Token>,
    at_line_start: bool,
    in_flow: bool,
    last_was_linebreak: bool,
    last_indent: usize,
    awaiting_line_content_start: bool,
    last_token_started_line: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_scan_tag() {
        let mut source = Buffer::new(b"!!str hello");
        let mut lexer = Lexer::new(&mut source, false);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Tag("!!str".to_string()));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("hello".to_string()));
    }

    #[test]
    fn test_scan_anchor() {
        let mut source = Buffer::new(b"&anchor value");
        let mut lexer = Lexer::new(&mut source, false);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Anchor("anchor".to_string()));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("value".to_string()));
    }

    #[test]
    fn test_scan_alias() {
        let mut source = Buffer::new(b"*anchor");
        let mut lexer = Lexer::new(&mut source, false);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Alias("anchor".to_string()));
    }

    #[test]
    fn test_flow_indicators() {
        let mut source = Buffer::new(b"{a: b}");
        // In flow context, braces should be treated as flow mapping
        // delimiters and not included in plain scalars.
        let mut lexer = Lexer::new(&mut source, true);

        assert_eq!(lexer.next().unwrap().unwrap(), Token::FlowMappingStart);
        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::Plain("a".to_string())
        );
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::Plain("b".to_string())
        );
        assert_eq!(lexer.next().unwrap().unwrap(), Token::FlowMappingEnd);
    }

    #[test]
    fn test_quoted_strings() {
        let mut source = Buffer::new(b"'single' \"double\"");
        let mut lexer = Lexer::new(&mut source, false);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::SingleQuoted("single".to_string())
        );
        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::DoubleQuoted("double".to_string(), false, usize::MAX)
        );
    }
    #[test]
    fn test_document_markers_empty() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"---\n...\n");
        let mut lexer = Lexer::new(&mut source, false);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentStart);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentEnd);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);
    }
    #[test]
    fn test_document_markers_with_content() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"---\nkey: value\n...");
        let mut lexer = Lexer::new(&mut source, false);

        // Document start
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentStart);

        // Newline
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        // Plain key
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("key".to_string()));

        // Colon
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Colon);

        // Plain value
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("value".to_string()));

        // Newline
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        // Document end
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentEnd);

        // EOF
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Eof);
    }
    #[test]
    fn test_document_markers_three() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"---\n...\n---\n...\n---\n...\n");
        let mut lexer = Lexer::new(&mut source, false);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentStart);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentEnd);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentStart);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentEnd);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentStart);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentEnd);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);
    }
}

// ---------------------------------------------------------------------------
// Additional impl blocks for Lexer — included directly into this module so
// all private struct fields remain accessible without visibility changes.
// ---------------------------------------------------------------------------
include!("block.rs");
include!("flow.rs");
include!("decorators.rs");
include!("strings.rs");
include!("scalars.rs");
