use crate::lexer_debug;

/**
 * YAML Lexer / Tokenizer
 *
 * Implements the tokenization layer for YAML parsing, sitting between the character source
 * and the parser. Pre-processes decorators (tags, anchors), identifies token boundaries,
 * and simplifies the parser by avoiding infinite lookahead loops.
 *
 * Copyright (c) 2026 YAML Library Developers
 */
use crate::constants::*;
use crate::io::traits::ISource;
use crate::parser::utils::error_helpers;
use crate::parser::utils::token_scan;

// Macro for debug logging in the lexer
// Helper for tag delimiter logic to avoid borrow checker issues
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
        if self.in_flow {
            // Suppress Token::Newline in flow context
            return self.scan_token();
        }
        Ok(Some(Token::Newline))
    }

    /// Scan until newline
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

    /// Peek ahead n characters
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

        // Plain key 'God'
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("key".to_string()));

        // Colon
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Colon);

        // Plain value '42'
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("value".to_string()));

        // Newline
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        // Document end
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::DocumentEnd);

        // Newline
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
