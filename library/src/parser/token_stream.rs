//! Token stream wrapper for parser integration
//!
//! This module provides a higher-level interface over the lexer, handling
//! common patterns like consuming decorators, looking for specific tokens,
//! and managing token sequences.

use crate::io::traits::ISource;
use crate::parser::directives::DirectiveContext;
use crate::parser::lexer::{Lexer, Token};

/// Decorators (tags and anchors) extracted from token stream
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Decorators {
    pub tag: Option<String>,
    pub anchor: Option<String>,
}

/// Token stream for high-level parser operations
pub struct TokenStream<'a> {
    lexer: Lexer<'a>,
    _directives: &'a DirectiveContext,
    // Track a simple position counter for progress checks
    position_counter: usize,
}

// Env-controlled logging for token stream internals
#[cfg(feature = "debug-trace")]
#[inline]
fn ts_log(msg: String) {
    #[cfg(feature = "std")]
    {
        if let Ok(v) = std::env::var("YAML_TRACE_TOKENS") {
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

#[allow(dead_code)]
impl<'a> TokenStream<'a> {
    /// Create a new token stream and load the first token
    ///
    /// Returns Result to propagate lexer errors (e.g., empty alias/anchor names)
    pub fn new(
        source: &'a mut dyn ISource,
        directives: &'a DirectiveContext,
        in_flow: bool,
    ) -> Result<Self, String> {
        let mut lexer = Lexer::new(source, in_flow);
        // Load the first token - propagate errors
        lexer.next()?;
        let ts = TokenStream {
            lexer,
            _directives: directives,
            position_counter: 0,
        };
        #[cfg(feature = "debug-trace")]
        ts_log(format!("token_stream: new -> current = {:?}", ts.current()));
        Ok(ts)
    }

    /// Get the current token without consuming it
    #[inline]
    pub fn current(&self) -> Option<&Token> {
        self.lexer.current()
    }

    /// Advance to the next token
    #[inline]
    pub fn next(&mut self) -> Result<Option<Token>, String> {
        let _prev = self.lexer.current().cloned();
        let out = self.lexer.next();
        if out.is_ok() {
            self.position_counter = self.position_counter.wrapping_add(1);
        }
        #[cfg(feature = "debug-trace")]
        if let Ok(ref _t) = out {
            ts_log(format!(
                "token_stream: next {:?} -> {:?}",
                _prev,
                self.lexer.current()
            ));
        }
        out
    }

    /// Returns a simple position counter for progress checks
    pub fn stream_position(&self) -> usize {
        self.position_counter
    }

    /// Peek at the next token without consuming it
    #[inline]
    pub fn peek(&mut self) -> Result<Option<&Token>, String> {
        let res = self.lexer.peek();
        #[cfg(feature = "debug-trace")]
        if let Ok(tok) = res {
            ts_log(format!("token_stream: peek -> {:?}", tok));
        }
        res
    }

    /// Check if current token matches a predicate
    #[inline]
    pub fn is_current<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&Token) -> bool,
    {
        self.current().map_or(false, predicate)
    }

    /// Expect a specific token and consume it
    #[inline]
    pub fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.current() {
            Some(token) if token == &expected => {
                self.next()?;
                Ok(())
            }
            Some(token) => Err(format!("Expected token {:?}, got {:?}", expected, token)),
            None => Err(format!("Expected token {:?}, got EOF", expected)),
        }
    }

    /// Skip whitespace tokens (newlines, indents)
    #[inline]
    pub fn skip_whitespace(&mut self) -> Result<(), String> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_whitespace at {:?}",
            self.current()
        ));
        while self
            .current()
            .map_or(false, |t| matches!(t, Token::Newline | Token::Indent(_)))
        {
            self.next()?;
        }
        Ok(())
    }

    /// Skip comments
    #[inline]
    pub fn skip_comments(&mut self) -> Result<(), String> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_comments at {:?}",
            self.current()
        ));
        while matches!(self.current(), Some(Token::Comment(_))) {
            self.next()?;
        }
        Ok(())
    }

    /// Skip whitespace and comments
    #[inline]
    pub fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_whitespace_and_comments at {:?}",
            self.current()
        ));
        while self.current().map_or(false, |t| Self::is_trivia(t)) {
            self.next()?;
        }
        Ok(())
    }

    #[inline]
    fn is_trivia(token: &Token) -> bool {
        matches!(token, Token::Newline | Token::Indent(_) | Token::Comment(_))
    }

    /// Consume decorators (tags and anchors) from the token stream
    ///
    /// This handles both orderings:
    /// - tag then anchor: `!!str &name`
    /// - anchor then tag: `&name !!str`
    ///
    /// Returns the decorators without resolving tag handles.
    pub fn consume_decorators(&mut self) -> Result<Decorators, String> {
        let mut decorators = Decorators::default();

        // Allow up to 2 passes to handle both tag and anchor
        // DON'T skip whitespace here - let caller decide if they need to skip before calling
        for _ in 0..2 {
            match self.current() {
                Some(Token::Tag(tag_str)) => {
                    if decorators.tag.is_some() {
                        return Err(crate::parser::document::error_builder::syntax_error(
                            self.source_mut(),
                            "Duplicate tag found"
                        ));
                    }
                    // Preserve raw tag handle; resolve later in value parsing
                    decorators.tag = Some(tag_str.clone());
                    self.next()?;
                }
                Some(Token::Anchor(name)) => {
                    if decorators.anchor.is_some() {
                        return Err(crate::parser::document::error_builder::syntax_error(
                            self.source_mut(),
                            "Duplicate anchor found"
                        ));
                    }
                    decorators.anchor = Some(name.clone());
                    self.next()?;
                }
                _ => break,
            }
        }

        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: consume_decorators -> {:?}",
            decorators
        ));
        Ok(decorators)
    }

    /// Check if we're at the start of a flow collection
    pub fn at_flow_start(&self) -> bool {
        matches!(
            self.current(),
            Some(Token::FlowMappingStart) | Some(Token::FlowSequenceStart)
        )
    }

    /// Check if we're at the start of a quoted string
    pub fn at_quoted_string(&self) -> bool {
        matches!(
            self.current(),
            Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(_))
        )
    }

    /// Check if we're at a sequence indicator
    pub fn at_sequence_indicator(&self) -> bool {
        matches!(self.current(), Some(Token::Dash))
    }

    /// Check if we're at end of stream
    pub fn at_eof(&self) -> bool {
        matches!(self.current(), Some(Token::Eof) | None)
    }

    /// Consume a plain scalar token
    pub fn consume_plain_scalar(&mut self) -> Result<String, String> {
        match self.current() {
            Some(Token::Plain(s)) => {
                let result = s.clone();
                self.next()?;
                Ok(result)
            }
            Some(token) => Err(format!("Expected plain scalar, got {:?}", token)),
            None => Err("Expected plain scalar, got EOF".to_string()),
        }
    }

    /// Consume a quoted scalar token (single or double quoted)
    pub fn consume_quoted_scalar(&mut self) -> Result<String, String> {
        match self.current() {
            Some(Token::SingleQuoted(s)) | Some(Token::DoubleQuoted(s)) => {
                let result = s.clone();
                self.next()?;
                Ok(result)
            }
            Some(token) => Err(format!("Expected quoted scalar, got {:?}", token)),
            None => Err("Expected quoted scalar, got EOF".to_string()),
        }
    }

    /// Consume any scalar token (plain, single quoted, or double quoted)
    pub fn consume_scalar(&mut self) -> Result<(String, ScalarType), String> {
        match self.current() {
            Some(Token::Plain(s)) => {
                let result = s.clone();
                self.next()?;
                Ok((result, ScalarType::Plain))
            }
            Some(Token::SingleQuoted(s)) => {
                let result = s.clone();
                self.next()?;
                Ok((result, ScalarType::SingleQuoted))
            }
            Some(Token::DoubleQuoted(s)) => {
                let result = s.clone();
                self.next()?;
                Ok((result, ScalarType::DoubleQuoted))
            }
            Some(token) => Err(format!("Expected scalar, got {:?}", token)),
            None => Err("Expected scalar, got EOF".to_string()),
        }
    }

    /// Get the current indentation level
    pub fn indent_level(&self) -> usize {
        self.lexer.indent_level()
    }

    /// Check if the next token (after whitespace) is a colon
    pub fn has_colon_ahead(&mut self) -> Result<bool, String> {
        // Save position
        let _current_state = self.current().cloned();

        // Skip whitespace
        while matches!(self.peek()?, Some(Token::Newline) | Some(Token::Indent(_))) {
            self.next()?;
        }

        // Check for colon
        let has_colon = matches!(self.peek()?, Some(Token::Colon));

        // Note: We've consumed tokens during lookahead
        // In a real implementation, we'd need a more sophisticated approach
        // For now, this is a simplified version

        #[cfg(feature = "debug-trace")]
        ts_log(format!("token_stream: has_colon_ahead -> {}", has_colon));
        Ok(has_colon)
    }

    /// Expose a mutable reference to the underlying source for error reporting
    pub fn source_mut(&mut self) -> &mut dyn crate::io::traits::ISource {
        self.lexer.source
    }
}

/// Type of scalar value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ScalarType {
    Plain,
    SingleQuoted,
    DoubleQuoted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_consume_decorators_tag_only() {
        let mut source = Buffer::new(b"!!str value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let decorators = stream.consume_decorators().unwrap();

        assert!(decorators.tag.is_some());
        assert_eq!(decorators.tag.unwrap(), "!!str");
        assert!(decorators.anchor.is_none());
    }

    #[test]
    fn test_consume_decorators_anchor_only() {
        let mut source = Buffer::new(b"&myanchor value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let decorators = stream.consume_decorators().unwrap();

        assert!(decorators.anchor.is_some());
        assert_eq!(decorators.anchor.unwrap(), "myanchor");
        assert!(decorators.tag.is_none());
    }

    #[test]
    fn test_consume_decorators_both() {
        let mut source = Buffer::new(b"!!str &myanchor value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let decorators = stream.consume_decorators().unwrap();

        assert!(decorators.tag.is_some());
        assert!(decorators.anchor.is_some());
        assert_eq!(decorators.tag.unwrap(), "!!str");
        assert_eq!(decorators.anchor.unwrap(), "myanchor");
    }

    #[test]
    fn test_consume_decorators_reversed() {
        let mut source = Buffer::new(b"&myanchor !!str value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let decorators = stream.consume_decorators().unwrap();

        assert!(decorators.tag.is_some());
        assert!(decorators.anchor.is_some());
        assert_eq!(decorators.tag.unwrap(), "!!str");
        assert_eq!(decorators.anchor.unwrap(), "myanchor");
    }

    #[test]
    fn test_skip_whitespace() {
        let mut source = Buffer::new(b"\n  \n  value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        stream.next().unwrap(); // Initialize
        stream.skip_whitespace().unwrap();

        assert!(matches!(stream.current(), Some(Token::Plain(_))));
    }

    #[test]
    fn test_document_markers() {
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
}
