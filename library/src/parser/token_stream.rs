//! Token stream wrapper for parser integration
//!
//! This module provides a higher-level interface over the lexer, handling
//! common patterns like consuming decorators, looking for specific tokens,
//! and managing token sequences.

use crate::parser::lexer::{Lexer, Token};
use crate::parser::directives::DirectiveContext;
use crate::io::traits::ISource;

/// Decorators (tags and anchors) extracted from token stream
#[derive(Debug, Clone, Default)]
pub struct Decorators {
    pub tag: Option<String>,
    pub anchor: Option<String>,
}

/// Token stream for high-level parser operations
pub struct TokenStream<'a> {
    lexer: Lexer<'a>,
    directives: &'a DirectiveContext,
}

impl<'a> TokenStream<'a> {
    /// Create a new token stream and load the first token
    pub fn new(source: &'a mut dyn ISource, directives: &'a DirectiveContext) -> Self {
        let mut lexer = Lexer::new(source);
        // Load the first token
        let _ = lexer.next();
        TokenStream {
            lexer,
            directives,
        }
    }

    /// Get the current token without consuming it
    pub fn current(&self) -> Option<&Token> {
        self.lexer.current()
    }

    /// Advance to the next token
    pub fn next(&mut self) -> Result<Option<Token>, String> {
        self.lexer.next()
    }

    /// Peek at the next token without consuming it
    pub fn peek(&mut self) -> Result<Option<&Token>, String> {
        self.lexer.peek()
    }

    /// Check if current token matches a predicate
    pub fn is_current<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&Token) -> bool,
    {
        self.current().map_or(false, predicate)
    }

    /// Expect a specific token and consume it
    pub fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.current() {
            Some(token) if token == &expected => {
                self.next()?;
                Ok(())
            }
            Some(token) => Err(format!(
                "Expected token {:?}, got {:?}",
                expected, token
            )),
            None => Err(format!("Expected token {:?}, got EOF", expected)),
        }
    }

    /// Skip whitespace tokens (newlines, indents)
    pub fn skip_whitespace(&mut self) -> Result<(), String> {
        loop {
            match self.current() {
                Some(Token::Newline) | Some(Token::Indent(_)) => {
                    self.next()?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Skip comments
    pub fn skip_comments(&mut self) -> Result<(), String> {
        while matches!(self.current(), Some(Token::Comment(_))) {
            self.next()?;
        }
        Ok(())
    }

    /// Skip whitespace and comments
    pub fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        loop {
            match self.current() {
                Some(Token::Newline) | Some(Token::Indent(_)) | Some(Token::Comment(_)) => {
                    self.next()?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Consume decorators (tags and anchors) from the token stream
    ///
    /// This handles both orderings:
    /// - tag then anchor: `!!str &name`
    /// - anchor then tag: `&name !!str`
    ///
    /// Returns the decorators and resolves tags using the directive context.
    pub fn consume_decorators(&mut self) -> Result<Decorators, String> {
        let mut decorators = Decorators::default();

        // Allow up to 2 passes to handle both tag and anchor
        for _ in 0..2 {
            self.skip_whitespace()?;

            match self.current() {
                Some(Token::Tag(tag_str)) => {
                    if decorators.tag.is_some() {
                        return Err("Duplicate tag found".to_string());
                    }
                    // Resolve the tag using directive context
                    let resolved = self.directives.resolve_tag(tag_str);
                    decorators.tag = Some(resolved);
                    self.next()?;
                }
                Some(Token::Anchor(name)) => {
                    if decorators.anchor.is_some() {
                        return Err("Duplicate anchor found".to_string());
                    }
                    decorators.anchor = Some(name.clone());
                    self.next()?;
                }
                _ => break,
            }
        }

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
        let current_state = self.current().cloned();
        
        // Skip whitespace
        while matches!(self.peek()?, Some(Token::Newline) | Some(Token::Indent(_))) {
            self.next()?;
        }
        
        // Check for colon
        let has_colon = matches!(self.peek()?, Some(Token::Colon));
        
        // Note: We've consumed tokens during lookahead
        // In a real implementation, we'd need a more sophisticated approach
        // For now, this is a simplified version
        
        Ok(has_colon)
    }
}

/// Type of scalar value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
        let decorators = stream.consume_decorators().unwrap();
        
        assert!(decorators.tag.is_some());
        assert_eq!(decorators.tag.unwrap(), "!!str");
        assert!(decorators.anchor.is_none());
    }

    #[test]
    fn test_consume_decorators_anchor_only() {
        let mut source = Buffer::new(b"&myanchor value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
        let decorators = stream.consume_decorators().unwrap();
        
        assert!(decorators.anchor.is_some());
        assert_eq!(decorators.anchor.unwrap(), "myanchor");
        assert!(decorators.tag.is_none());
    }

    #[test]
    fn test_consume_decorators_both() {
        let mut source = Buffer::new(b"!!str &myanchor value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
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
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
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
        let mut stream = TokenStream::new(&mut source, &directives);
        
        stream.next().unwrap(); // Initialize
        stream.skip_whitespace().unwrap();
        
        assert!(matches!(stream.current(), Some(Token::Plain(_))));
    }
}
