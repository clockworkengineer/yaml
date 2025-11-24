//! Lexer/Tokenizer for YAML parsing
//!
//! This module provides a tokenization layer that sits between the character source
//! and the parser. It pre-processes decorators (tags, anchors) and identifies token
//! boundaries, making the parser simpler and avoiding infinite loops in lookahead.

use crate::constants::*;
use crate::io::traits::ISource;

/// A YAML token with its type and associated data
#[derive(Debug, Clone, PartialEq)]
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
    source: &'a mut dyn ISource,
    current_token: Option<Token>,
    peeked_token: Option<Token>,
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer wrapping a character source
    pub fn new(source: &'a mut dyn ISource) -> Self {
        Lexer {
            source,
            current_token: None,
            peeked_token: None,
            at_line_start: true,
        }
    }

    /// Get the current token without consuming it
    pub fn current(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    /// Advance to the next token
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
            None => return Ok(Some(Token::Eof)),
        };

        // Handle line-start indentation
        if self.at_line_start && (ch == CHAR_SPACE || ch == CHAR_TAB) {
            let indent = self.scan_indentation()?;
            self.at_line_start = false;
            return Ok(Some(Token::Indent(indent)));
        }

        self.at_line_start = false;

        // Match token types
        match ch {
            CHAR_NEWLINE => {
                self.source.next();
                self.at_line_start = true;
                Ok(Some(Token::Newline))
            }
            CHAR_CARRIAGE_RETURN => {
                self.source.next();
                if self.source.current() == Some(CHAR_NEWLINE) {
                    self.source.next();
                }
                self.at_line_start = true;
                Ok(Some(Token::Newline))
            }
            CHAR_HASH => {
                self.source.next();
                let comment = self.scan_until_newline();
                Ok(Some(Token::Comment(comment)))
            }
            '!' => Ok(Some(self.scan_tag()?)),
            '&' => Ok(Some(self.scan_anchor()?)),
            CHAR_ASTERISK => Ok(Some(self.scan_alias()?)),
            CHAR_LBRACE => {
                self.source.next();
                Ok(Some(Token::FlowMappingStart))
            }
            CHAR_RBRACE => {
                self.source.next();
                Ok(Some(Token::FlowMappingEnd))
            }
            CHAR_LBRACKET => {
                self.source.next();
                Ok(Some(Token::FlowSequenceStart))
            }
            CHAR_RBRACKET => {
                self.source.next();
                Ok(Some(Token::FlowSequenceEnd))
            }
            CHAR_COMMA => {
                self.source.next();
                Ok(Some(Token::Comma))
            }
            CHAR_COLON => {
                self.source.next();
                Ok(Some(Token::Colon))
            }
            '-' => {
                // Could be: dash (sequence), document start (---), or plain scalar
                let state = self.source.save_state();
                self.source.next();

                match (self.source.current(), self.peek_ahead(2)) {
                    (Some('-'), Some('-')) => {
                        // Document start: ---
                        self.source.next();
                        self.source.next();
                        Ok(Some(Token::DocumentStart))
                    }
                    (Some(c), _) if c.is_whitespace() || c == CHAR_NEWLINE => {
                        // Dash (sequence indicator)
                        Ok(Some(Token::Dash))
                    }
                    _ => {
                        // Plain scalar starting with dash
                        self.source.restore_state(state);
                        Ok(Some(self.scan_plain_scalar()?))
                    }
                }
            }
            '.' => {
                // Could be: document end (...) or plain scalar
                let state = self.source.save_state();
                self.source.next();

                match (self.source.current(), self.peek_ahead(2)) {
                    (Some('.'), Some('.')) => {
                        // Document end: ...
                        self.source.next();
                        self.source.next();
                        Ok(Some(Token::DocumentEnd))
                    }
                    _ => {
                        // Plain scalar starting with dot
                        self.source.restore_state(state);
                        Ok(Some(self.scan_plain_scalar()?))
                    }
                }
            }
            '?' => {
                self.source.next();
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
    fn scan_indentation(&mut self) -> Result<usize, String> {
        let mut count = 0;
        while let Some(ch) = self.source.current() {
            if ch == CHAR_SPACE {
                count += 1;
                self.source.next();
            } else if ch == CHAR_TAB {
                return Err("Tabs cannot be used for indentation in YAML".to_string());
            } else {
                break;
            }
        }
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
            tag_name.push(ch);
            self.source.next();
        }

        let tag = if is_double {
            format!("!!{}", tag_name)
        } else {
            format!("!{}", tag_name)
        };

        if tag == "!" || tag == "!!" {
            return Err("Empty tag".to_string());
        }

        Ok(Token::Tag(tag))
    }

    /// Scan an anchor: &name
    fn scan_anchor(&mut self) -> Result<Token, String> {
        self.source.next(); // consume '&'

        let mut name = String::new();
        while let Some(ch) = self.source.current() {
            // Per YAML spec, anchor names are ns-anchor-char
            // Stop at: whitespace, newlines, comments, and flow indicators []{},
            // Colons are allowed in anchor names
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
            return Err("Empty anchor name".to_string());
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
            return Err("Empty alias name".to_string());
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
                None => return Err("Unclosed single-quoted string".to_string()),
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
                        Some('n') => content.push('\n'),
                        Some('t') => content.push('\t'),
                        Some('r') => content.push('\r'),
                        Some('\\') => content.push('\\'),
                        Some('"') => content.push('"'),
                        Some(c) => content.push(c),
                        None => return Err("Unclosed double-quoted string".to_string()),
                    }
                    self.source.next();
                }
                Some(CHAR_DOUBLE_QUOTE) => {
                    self.source.next();
                    break;
                }
                Some(ch) => {
                    content.push(ch);
                    self.source.next();
                }
                None => return Err("Unclosed double-quoted string".to_string()),
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
        let mut lexer = Lexer::new(&mut source);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Tag("!!str".to_string()));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("hello".to_string()));
    }

    #[test]
    fn test_scan_anchor() {
        let mut source = Buffer::new(b"&anchor value");
        let mut lexer = Lexer::new(&mut source);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Anchor("anchor".to_string()));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Plain("value".to_string()));
    }

    #[test]
    fn test_scan_alias() {
        let mut source = Buffer::new(b"*anchor");
        let mut lexer = Lexer::new(&mut source);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Alias("anchor".to_string()));
    }

    #[test]
    fn test_flow_indicators() {
        let mut source = Buffer::new(b"{a: b}");
        let mut lexer = Lexer::new(&mut source);

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
        let mut lexer = Lexer::new(&mut source);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::SingleQuoted("single".to_string())
        );
        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::DoubleQuoted("double".to_string())
        );
    }
}
