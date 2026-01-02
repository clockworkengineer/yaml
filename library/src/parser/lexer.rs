//! Lexer/Tokenizer for YAML parsing
//!
//! This module provides a tokenization layer that sits between the character source
//! and the parser. It pre-processes decorators (tags, anchors) and identifies token
//! boundaries, making the parser simpler and avoiding infinite loops in lookahead.

use crate::constants::*;
use crate::io::traits::ISource;

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
    DoubleQuoted(String),

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
        }
    }

    /// Get the current token without consuming it
    #[inline]
    pub fn current(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    /// Advance to the next token
    #[inline]
    pub fn next(&mut self) -> Result<Option<Token>, String> {
        if let Some(peeked) = self.peeked_token.take() {
            self.current_token = Some(peeked.clone());
            return Ok(Some(peeked));
        }

        let token = self.scan_token()?;
        self.current_token = token.clone();
        Ok(token)
    }

    /// Peek at the next token without consuming it
    #[allow(dead_code)]
    #[inline]
    pub fn peek(&mut self) -> Result<Option<&Token>, String> {
        if self.peeked_token.is_none() {
            self.peeked_token = self.scan_token()?;
        }
        Ok(self.peeked_token.as_ref())
    }

    /// Scan and return the next token from the source
    fn scan_token(&mut self) -> Result<Option<Token>, String> {
        // Skip whitespace (except at line start where it's indentation)
        if !self.at_line_start {
            self.skip_horizontal_whitespace();
        }

        let ch = match self.source.current() {
            Some(c) => c,
            None => {
                println!("LEXER TRACE: Emitting Token::Eof");
                return Ok(Some(Token::Eof));
            }
        };

        // Handle line-start indentation
        if self.at_line_start {
            if ch == CHAR_SPACE || ch == CHAR_TAB {
                let indent = self.scan_indentation(self.in_flow)?;
                self.at_line_start = false;
                self.last_indent = indent;
                println!("LEXER TRACE: Emitting Token::Indent({})", indent);
                return Ok(Some(Token::Indent(indent)));
            } else if self.last_indent > 0 {
                // No leading space/tab, but previous indent was > 0: emit Indent(0) to signal dedent
                self.at_line_start = false;
                self.last_indent = 0;
                println!("LEXER TRACE: Emitting Token::Indent(0) (dedent to top level)");
                return Ok(Some(Token::Indent(0)));
            }
        }

        self.at_line_start = false;

        // Match token types
        match ch {
            CHAR_NEWLINE => {
                println!(
                    "LEXER TRACE: Emitting Token::Newline (in_flow={})",
                    self.in_flow
                );
                self.source.next();
                self.at_line_start = true;
                self.last_was_linebreak = true;
                if self.in_flow {
                    // Suppress Token::Newline in flow context
                    return self.scan_token();
                }
                Ok(Some(Token::Newline))
            }
            CHAR_CARRIAGE_RETURN => {
                println!(
                    "LEXER TRACE: Emitting Token::Newline (CR) (in_flow={})",
                    self.in_flow
                );
                self.source.next();
                if self.source.current() == Some(CHAR_NEWLINE) {
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
            CHAR_HASH => {
                self.source.next();
                let comment = self.scan_until_newline();
                Ok(Some(Token::Comment(comment)))
            }
            '!' => {
                println!("LEXER TRACE: Emitting Token::Tag");
                Ok(Some(self.scan_tag()?))
            }
            '&' => {
                println!("LEXER TRACE: Emitting Token::Anchor");
                Ok(Some(self.scan_anchor()?))
            }
            CHAR_ASTERISK => {
                println!("LEXER TRACE: Emitting Token::Alias");
                Ok(Some(self.scan_alias()?))
            }
            CHAR_LBRACE => {
                self.source.next();
                self.last_was_linebreak = false;
                println!("LEXER TRACE: Emitting Token::FlowMappingStart");
                Ok(Some(Token::FlowMappingStart))
            }
            CHAR_RBRACE => {
                println!("LEXER TRACE: Emitting Token::FlowMappingEnd");
                self.source.next();
                // Allow any horizontal whitespace before comment
                let mut seen_whitespace = false;
                while let Some(c) = self.source.current() {
                    if c == ' ' || c == '\t' {
                        seen_whitespace = true;
                        self.source.next();
                    } else {
                        break;
                    }
                }
                if let Some('#') = self.source.current() {
                    if !seen_whitespace {
                        return Err(crate::parser::document::error_builder::syntax_error(
                            self.source,
                            "YAML syntax error: comment must be preceded by whitespace after '}' in flow collection"
                        ));
                    }
                }
                // Validate what comes after } - must be whitespace, newline, flow indicator, or EOF
                if let Some(c) = self.source.current() {
                    if !seen_whitespace && c != '\n' && c != '\r' && c != ',' && c != ']' && c != '}' && c != '#' && c != ':' {
                        // Check if it's alphanumeric which clearly indicates invalid
                        if c.is_alphanumeric() {
                            return Err(format!("YAML syntax error: Invalid content '{}' immediately after '}}' - whitespace or newline required", c));
                        }
                    }
                }
                self.last_was_linebreak = false;
                Ok(Some(Token::FlowMappingEnd))
            }
            CHAR_LBRACKET => {
                self.source.next();
                self.last_was_linebreak = false;
                Ok(Some(Token::FlowSequenceStart))
            }
            CHAR_RBRACKET => {
                self.source.next();
                // Allow any horizontal whitespace before comment
                let mut seen_whitespace = false;
                while let Some(c) = self.source.current() {
                    if c == ' ' || c == '\t' {
                        seen_whitespace = true;
                        self.source.next();
                    } else {
                        break;
                    }
                }
                if let Some('#') = self.source.current() {
                    if !seen_whitespace {
                        return Err(crate::parser::document::error_builder::syntax_error(
                            self.source,
                            "YAML syntax error: comment must be preceded by whitespace after ']' in flow collection"
                        ));
                    }
                }
                // Validate what comes after ] - must be whitespace, newline, flow indicator, or EOF
                if let Some(c) = self.source.current() {
                    if !seen_whitespace && c != '\n' && c != '\r' && c != ',' && c != ']' && c != '}' && c != '#' && c != ':' {
                        // Check if it's alphanumeric which clearly indicates invalid
                        if c.is_alphanumeric() {
                            return Err(format!("YAML syntax error: Invalid content '{}' immediately after ']' - whitespace or newline required", c));
                        }
                    }
                }
                self.last_was_linebreak = false;
                Ok(Some(Token::FlowSequenceEnd))
            }
            CHAR_COMMA => {
                self.source.next();
                // Allow any horizontal whitespace before comment
                let mut seen_whitespace = false;
                while let Some(c) = self.source.current() {
                    if c == ' ' || c == '\t' {
                        seen_whitespace = true;
                        self.source.next();
                    } else {
                        break;
                    }
                }
                if let Some('#') = self.source.current() {
                    if !seen_whitespace {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "DEBUG: Comment after comma with no whitespace at {:?}",
                            self.source.current()
                        );
                        return Err(crate::parser::document::error_builder::syntax_error(
                            self.source,
                            "YAML syntax error: comment must be preceded by whitespace after ',' in flow collection"
                        ));
                    }
                }
                self.last_was_linebreak = false;
                Ok(Some(Token::Comma))
            }
            CHAR_COLON => {
                self.source.next();
                self.last_was_linebreak = false;
                Ok(Some(Token::Colon))
            }
            '-' => {
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
            '.' => {
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
            '?' => {
                self.source.next();
                self.last_was_linebreak = false;
                Ok(Some(Token::QuestionMark))
            }
            '%' => Ok(Some(self.scan_directive()?)),
            CHAR_SINGLE_QUOTE => Ok(Some(self.scan_single_quoted()?)),
            CHAR_DOUBLE_QUOTE => Ok(Some(self.scan_double_quoted()?)),
            _ => Ok(Some(self.scan_plain_scalar()?)),
        }
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

    /// Scan indentation at line start
    fn scan_indentation(&mut self, in_flow: bool) -> Result<usize, String> {
        let mut count = 0;
        let mut tab_at_start = false;
        let first_char_is_tab = self.source.current() == Some(CHAR_TAB);

        while let Some(ch) = self.source.current() {
            if ch == CHAR_SPACE {
                count += 1;
                self.source.next();
            } else if ch == CHAR_TAB {
                // Tabs are forbidden for indentation in YAML block context
                // But allowed as whitespace in flow context (inside [], {})
                if in_flow {
                    // In flow context, tabs are just whitespace - skip them
                    self.source.next();
                    continue;
                }

                // Check if this is a tab at the very start (primary indentation)
                if count == 0 && first_char_is_tab {
                    tab_at_start = true;
                }

                // In block context, reject tabs as primary indentation
                // (tab at column 0 or before any spaces)
                if tab_at_start {
                    // Tab as primary indentation in block context - error
                    return Err(crate::parser::document::error_builder::forbidden_error(
                        self.source,
                        "Tabs",
                        "as indentation in YAML"
                    ));
                }

                // Tab after spaces: allow it through (e.g., in block scalar content)
                // Treat tab as advancing by 1 for indent counting purposes
                count += 1;
                self.source.next();
            } else {
                break;
            }
        }
        // Indentation consumed; reset linebreak flag
        self.last_was_linebreak = false;
        Ok(count)
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

    /// Scan a tag: !tag or !!tag
    fn scan_tag(&mut self) -> Result<Token, String> {
        self.source.next(); // consume '!'

        let is_double = if self.source.current() == Some('!') {
            self.source.next();
            true
        } else {
            false
        };

        let mut tag_name = String::new();
        while let Some(ch) = self.source.current() {
            // Allow colon in tag name unless it is followed by whitespace (mapping key separator)
            if ch == CHAR_COLON {
                // Peek ahead: if next char is whitespace or end, stop tag
                let state = self.source.save_state();
                self.source.next();
                let next_ch = self.source.current();
                self.source.restore_state(state);
                if next_ch.map_or(true, |c| c.is_whitespace() || c == CHAR_NEWLINE) {
                    break;
                }
            }
            if ch.is_whitespace()
                || ch == CHAR_NEWLINE
                || ch == CHAR_HASH
                || ch == CHAR_COMMA
                || ch == CHAR_LBRACKET
                || ch == CHAR_RBRACKET
                || ch == CHAR_LBRACE
                || ch == CHAR_RBRACE
            {
                break;
            }
            tag_name.push(ch);
            self.source.next();
        }

        let tag = if is_double {
            format!("!!{}", tag_name)
        } else {
            format!("!{}", tag_name)
        };

        // Note: "!" is a valid non-specific tag in YAML 1.2
        // It means the application should auto-detect the type
        // Similarly "!!" without a name is also technically valid (though unusual)

        Ok(Token::Tag(tag))
    }

    /// Scan an anchor: &name
    fn scan_anchor(&mut self) -> Result<Token, String> {
        self.source.next(); // consume '&'

        let mut name = String::new();
        while let Some(ch) = self.source.current() {
            // Per YAML spec, anchor names are ns-anchor-char
            // Stop at: whitespace, newlines, comments, flow indicators []{},
            // and colon. We treat ':' as a separator here so that '&name:'
            // is tokenized as Anchor("name") followed by Colon.
            if ch.is_whitespace()
                || ch == CHAR_NEWLINE
                || ch == CHAR_HASH
                || ch == CHAR_COMMA
                || ch == CHAR_LBRACKET
                || ch == CHAR_RBRACKET
                || ch == CHAR_LBRACE
                || ch == CHAR_RBRACE
                || ch == CHAR_COLON
            {
                break;
            }
            name.push(ch);
            self.source.next();
        }

        if name.is_empty() {
            return Err(crate::parser::document::error_builder::syntax_error(
                self.source,
                "Empty anchor name"
            ));
        }

        Ok(Token::Anchor(name))
    }

    /// Scan an alias: *name
    fn scan_alias(&mut self) -> Result<Token, String> {
        self.source.next(); // consume '*'

        let mut name = String::new();
        while let Some(ch) = self.source.current() {
            if ch.is_whitespace()
                || ch == CHAR_NEWLINE
                || ch == CHAR_HASH
                || ch == CHAR_COMMA
                || ch == CHAR_LBRACKET
                || ch == CHAR_RBRACKET
                || ch == CHAR_LBRACE
                || ch == CHAR_RBRACE
            {
                break;
            }
            name.push(ch);
            self.source.next();
        }

        if name.is_empty() {
            return Err(crate::parser::document::error_builder::syntax_error(
                self.source,
                "Empty alias name"
            ));
        }

        Ok(Token::Alias(name))
    }

    /// Scan a directive: %YAML or %TAG
    fn scan_directive(&mut self) -> Result<Token, String> {
        self.source.next(); // consume '%'

        let content = self.scan_until_newline();
        Ok(Token::Directive(content.trim().to_string()))
    }

    /// Scan a single-quoted scalar
    fn scan_single_quoted(&mut self) -> Result<Token, String> {
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
                        break;
                    }
                }
                Some(ch) => {
                    content.push(ch);
                    self.source.next();
                }
                None => {
                    #[cfg(debug_assertions)]
                    eprintln!("DEBUG: Unterminated single-quoted string: reached EOF");
                    return Err(crate::parser::document::error_builder::syntax_error(
                        self.source,
                        "YAML compliance error: Unterminated single-quoted string (unexpected EOF)"
                    ));
                }
            }
        }

        Ok(Token::SingleQuoted(content))
    }

    /// Scan a double-quoted scalar
    fn scan_double_quoted(&mut self) -> Result<Token, String> {
        self.source.next(); // consume opening quote

        let mut content = String::new();

        loop {
            match self.source.current() {
                Some('\\') => {
                    self.source.next();
                    match self.source.current() {
                        Some('0') => { content.push('\0'); self.source.next(); }
                        Some('a') => { content.push('\x07'); self.source.next(); }
                        Some('b') => { content.push('\x08'); self.source.next(); }
                        Some('t') | Some('\t') => { content.push('\t'); self.source.next(); }
                        Some('n') => { content.push('\n'); self.source.next(); }
                        Some('v') => { content.push('\x0B'); self.source.next(); }
                        Some('f') => { content.push('\x0C'); self.source.next(); }
                        Some('r') => { content.push('\r'); self.source.next(); }
                        Some('e') => { content.push('\x1B'); self.source.next(); }
                        Some(' ') => { content.push(' '); self.source.next(); }
                        Some('"') => { content.push('"'); self.source.next(); }
                        Some('/') => { content.push('/'); self.source.next(); }
                        Some('\\') => { content.push('\\'); self.source.next(); }
                        Some('N') => { content.push('\u{0085}'); self.source.next(); }
                        Some('_') => { content.push('\u{00A0}'); self.source.next(); }
                        Some('L') => { content.push('\u{2028}'); self.source.next(); }
                        Some('P') => { content.push('\u{2029}'); self.source.next(); }
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
                                    _ => return Err(crate::parser::document::error_builder::syntax_error(
                                        self.source,
                                        "YAML compliance error: Invalid \\x escape sequence, expected 2 hex digits"
                                    )),
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
                                    _ => return Err(crate::parser::document::error_builder::syntax_error(
                                        self.source,
                                        "YAML compliance error: Invalid \\u escape sequence, expected 4 hex digits"
                                    )),
                                }
                            }
                            let code = u32::from_str_radix(&hex, 16).unwrap();
                            match char::from_u32(code) {
                                Some(ch) => content.push(ch),
                                None => return Err(crate::parser::document::error_builder::syntax_error(
                                    self.source,
                                    &format!("YAML compliance error: Invalid unicode codepoint U+{:04X}", code)
                                )),
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
                                    _ => return Err(crate::parser::document::error_builder::syntax_error(
                                        self.source,
                                        "YAML compliance error: Invalid \\U escape sequence, expected 8 hex digits"
                                    )),
                                }
                            }
                            let code = u32::from_str_radix(&hex, 16).unwrap();
                            match char::from_u32(code) {
                                Some(ch) => content.push(ch),
                                None => return Err(format!("YAML compliance error: Invalid unicode codepoint U+{:08X}", code)),
                            }
                        }
                        // Invalid escape sequences - reject per YAML 1.2 spec
                        Some(c) => {
                            return Err(format!(
                                "YAML compliance error: Invalid escape sequence '\\{}' in double-quoted string",
                                c
                            ));
                        }
                        None => {
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "DEBUG: Unterminated double-quoted string: reached EOF after escape"
                            );
                            return Err("YAML compliance error: Unterminated double-quoted string (unexpected EOF after escape)".to_string());
                        }
                    }
                }
                Some(CHAR_DOUBLE_QUOTE) => {
                    self.source.next();
                    break;
                }
                Some(ch) => {
                    content.push(ch);
                    self.source.next();
                }
                None => {
                    #[cfg(debug_assertions)]
                    eprintln!("DEBUG: Unterminated double-quoted string: reached EOF");
                    return Err(
                        "YAML compliance error: Unterminated double-quoted string (unexpected EOF)"
                            .to_string(),
                    );
                }
            }
        }

        Ok(Token::DoubleQuoted(content))
    }

    /// Scan a plain (unquoted) scalar
    fn scan_plain_scalar(&mut self) -> Result<Token, String> {
        let mut content = String::new();

        loop {
            match self.source.current() {
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
                    // Newline, tab, etc. - end of scalar
                    break;
                }
                Some(ch)
                    if ch == CHAR_COMMA
                        || ch == CHAR_LBRACKET
                        || ch == CHAR_RBRACKET
                        || ch == CHAR_LBRACE
                        || ch == CHAR_RBRACE =>
                {
                    // Flow indicators - end of scalar
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
    #[allow(dead_code)]
    pub fn indent_level(&self) -> usize {
        self.source.get_current_indent_level()
    }
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
        let mut lexer = Lexer::new(&mut source, false);

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
            Token::DoubleQuoted("double".to_string())
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
