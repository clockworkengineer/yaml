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
    // Track current flow collection nesting depth for instrumentation
    flow_depth: i32,
    // Track the last token that was consumed; useful for distinguishing
    // standalone comment lines from inline comments.
    last_token: Option<Token>,
    // Line tracking: current logical line index (increments on Newline)
    current_line_index: usize,
    // Line index of the last consumed content scalar (Plain/Quoted)
    last_content_line_index: Option<usize>,
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

impl<'a> TokenStream<'a> {
    /// Create a new token stream and load the first token
    ///
    /// Returns Result to propagate lexer errors (e.g., empty alias/anchor names)
    pub fn new(
        source: &'a mut dyn ISource,
        directives: &'a DirectiveContext,
        in_flow: bool,
    ) -> Result<Self, crate::error::YamlError> {
        let mut lexer = Lexer::new(source, in_flow);
        // Load the first token - propagate errors
        lexer.next()?;
        let ts = TokenStream {
            lexer,
            _directives: directives,
            position_counter: 0,
            // Flow depth is tracked only for instrumentation; it starts at 0
            // and is updated as we consume tokens via `next()`.
            flow_depth: 0,
            last_token: None,
            current_line_index: 0,
            last_content_line_index: None,
        };
        #[cfg(feature = "debug-trace")]
        ts_log(format!("token_stream: new -> current = {:?}", ts.current()));
        Ok(ts)
    }

    /// Returns true if the token stream is currently in a flow collection
    /// context (i.e., inside [] or {}), based on the tracked flow_depth.
    #[inline]
    pub fn in_flow(&self) -> bool {
        self.flow_depth > 0
    }

    /// Get the current token without consuming it
    #[inline]
    pub fn current(&self) -> Option<&Token> {
        self.lexer.current()
    }

    /// Advance to the next token
    #[inline]
    pub fn next(&mut self) -> Result<Option<Token>, crate::error::YamlError> {
        // Capture the token we are about to consume so we can update
        // flow depth *before* lexing the next token. This ensures that
        // the lexer sees the correct `in_flow` context when scanning
        // content inside flow collections, which is important for
        // handling newlines and ':' correctly in cases like 5MUD.
        let prev = self.lexer.current().cloned();

        if let Some(tok) = prev.as_ref() {
            match tok {
                Token::FlowMappingStart | Token::FlowSequenceStart => {
                    // Entering a flow collection; subsequent tokens
                    // should be scanned in flow context.
                    self.flow_depth = self.flow_depth.saturating_add(1);
                }
                Token::FlowMappingEnd | Token::FlowSequenceEnd => {
                    // Leaving a flow collection; if depth reaches 0,
                    // revert to block (non-flow) context.
                    self.flow_depth = (self.flow_depth - 1).max(0);
                }
                Token::Newline => {
                    // Advance logical line index when consuming a newline.
                    self.current_line_index = self.current_line_index.saturating_add(1);
                }
                Token::Plain(_)
                | Token::SingleQuoted(_)
                | Token::DoubleQuoted(_) => {
                    // Record the line index of the last content scalar.
                    self.last_content_line_index = Some(self.current_line_index);
                }
                Token::DocumentStart | Token::DocumentEnd => {
                    // Reset content association at document boundaries.
                    self.last_content_line_index = None;
                }
                _ => {}
            }
        }

        // Propagate flow state to the lexer *before* fetching the next
        // token so that its scanning rules match the current context.
        self.lexer.set_in_flow(self.flow_depth > 0);

        let out = self.lexer.next();
        if out.is_ok() {
            self.position_counter = self.position_counter.wrapping_add(1);
            // Remember the token we just consumed so helpers like
            // skip_newlines_and_comments_with_flag can inspect the
            // context of comment tokens (standalone vs inline).
            self.last_token = prev;
        }
        #[cfg(feature = "debug-trace")]
        if let Ok(ref _t) = out {
            ts_log(format!(
                "token_stream: next {:?} -> {:?}",
                prev,
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
    pub fn peek(&mut self) -> Result<Option<&Token>, crate::error::YamlError> {
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
    pub fn expect(&mut self, expected: Token) -> Result<(), crate::error::YamlError> {
        match self.current() {
            Some(token) if token == &expected => {
                self.next()?;
                Ok(())
            }
            Some(_token) => Err(crate::parser::document::token_errors::expected_specific_token(
                self.source_mut(),
                expected.clone(),
            )),
            None => Err(crate::parser::document::token_errors::expected_specific_token(
                self.source_mut(),
                expected.clone(),
            )),
        }
    }

    /// If the current token matches `expected`, consume it and return true; otherwise return false.
    #[inline]
    pub fn consume_if(&mut self, expected: Token) -> Result<bool, crate::error::YamlError> {
        match self.current() {
            Some(token) if token == &expected => {
                self.next()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Internal DRY helper: advance while predicate matches current token
    #[inline]
    fn advance_while(
        &mut self,
        mut predicate: impl FnMut(&Token) -> bool,
    ) -> Result<(), crate::error::YamlError> {
        while self.current().map_or(false, |t| predicate(t)) {
            self.next()?;
        }
        Ok(())
    }

    /// Skip whitespace tokens (newlines, indents)
    #[inline]
    #[allow(dead_code)]
    pub fn skip_whitespace(&mut self) -> Result<(), crate::error::YamlError> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_whitespace at {:?}",
            self.current()
        ));
        self.advance_while(|t| matches!(t, Token::Newline | Token::Indent(_)))
    }

    /// Skip comments
    #[inline]
    #[allow(dead_code)]
    pub fn skip_comments(&mut self) -> Result<(), crate::error::YamlError> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_comments at {:?}",
            self.current()
        ));
        self.advance_while(|t| matches!(t, Token::Comment(_)))
    }

    /// Skip whitespace and comments
    #[inline]
    #[allow(dead_code)]
    pub fn skip_whitespace_and_comments(&mut self) -> Result<(), crate::error::YamlError> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_whitespace_and_comments at {:?}",
            self.current()
        ));
        self.advance_while(Self::is_trivia)
    }

    /// Alias for skipping all trivia (whitespace + comments) to encourage DRY usage
    #[inline]
    #[allow(dead_code)]
    pub fn skip_trivia(&mut self) -> Result<(), crate::error::YamlError> {
        self.skip_whitespace_and_comments()
    }

    /// Skip only newlines and comments, preserving `Indent` tokens for dedent detection.
    #[inline]
    #[allow(dead_code)]
    pub fn skip_newlines_and_comments(&mut self) -> Result<(), crate::error::YamlError> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_newlines_and_comments at {:?}",
            self.current()
        ));
        self.advance_while(|t| matches!(t, Token::Newline | Token::Comment(_)))
    }

    /// Skip newlines and comments, returning true if at least one
    /// *standalone* comment token was encountered (i.e., a comment that
    /// appears on its own line, not as an inline trailing comment).
    ///
    /// This is useful for callers (such as the block mapping parser) that
    /// need to distinguish between a simple blank-line separation, an
    /// inline comment at the end of a content line, and a comment line
    /// appearing before an indented block (as in 8XDJ).
    #[inline]
    #[allow(dead_code)]
    pub fn skip_newlines_and_comments_with_flag(
        &mut self,
    ) -> Result<bool, crate::error::YamlError> {
        #[cfg(feature = "debug-trace")]
        ts_log(format!(
            "token_stream: skip_newlines_and_comments_with_flag at {:?}",
            self.current()
        ));
        let mut saw_comment = false;
        while let Some(tok) = self.current() {
            match tok {
                Token::Newline => {
                    self.next()?;
                }
                Token::Comment(_) => {
                    // Treat this as a standalone comment only if the
                    // previous token was a line boundary or indent,
                    // not regular content. This prevents inline
                    // comments like `key: value # comment` from being
                    // mistaken for the 8XDJ-style pattern where a
                    // comment line sits between a scalar and an
                    // indented block.
                    match self.last_token {
                        None
                        | Some(Token::Newline)
                        | Some(Token::Indent(_))
                        | Some(Token::DocumentStart)
                        | Some(Token::DocumentEnd) => {
                            saw_comment = true;
                        }
                        _ => {}
                    }
                    self.next()?;
                }
                _ => break,
            }
        }
        Ok(saw_comment)
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
    pub fn consume_decorators(&mut self) -> Result<Decorators, crate::error::YamlError> {
        let mut decorators = Decorators::default();

        // Allow up to 2 passes to handle both tag and anchor
        // DON'T skip whitespace here - let caller decide if they need to skip before calling
        for _ in 0..2 {
            match self.current() {
                Some(Token::Tag(tag_str)) => {
                    if decorators.tag.is_some() {
                        return Err(crate::parser::document::token_errors::duplicate_tag_found(
                            self.source_mut(),
                        ));
                    }
                    // Validate explicit tag handle usage against current document directives.
                    if let Err(e) = self._directives.validate_tag_handle_usage(tag_str.as_str()) {
                        return Err(
                            crate::parser::document::token_errors::invalid_tag_handle_usage(
                                self.source_mut(),
                                &e.to_string(),
                            ),
                        );
                    }
                    // Preserve raw tag handle; resolve later in value parsing
                    decorators.tag = Some(tag_str.clone());
                    self.next()?;
                }
                Some(Token::Anchor(name)) => {
                    if decorators.anchor.is_some() {
                        return Err(
                            crate::parser::document::token_errors::duplicate_anchor_found(
                                self.source_mut(),
                            ),
                        );
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
    #[allow(dead_code)]
    pub fn at_flow_start(&self) -> bool {
        matches!(
            self.current(),
            Some(Token::FlowMappingStart) | Some(Token::FlowSequenceStart)
        )
    }

    /// Check if we're at the start of a quoted string
    #[allow(dead_code)]
    pub fn at_quoted_string(&self) -> bool {
        matches!(
            self.current(),
            Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(_))
        )
    }

    /// Check if we're at a sequence indicator
    #[allow(dead_code)]
    pub fn at_sequence_indicator(&self) -> bool {
        matches!(self.current(), Some(Token::Dash))
    }

    /// Check if we're at end of stream
    #[allow(dead_code)]
    pub fn at_eof(&self) -> bool {
        matches!(self.current(), Some(Token::Eof) | None)
    }

    /// Returns true if the current token is a `:` that appears on the
    /// same logical line as the previously consumed content token.
    ///
    /// This relies on `last_token` tracking and treats any intervening
    /// line boundary tokens (`Newline`, `Indent`) as evidence that the
    /// colon is on a subsequent line. Document markers also reset the
    /// line association.
    #[inline]
    pub fn is_colon_on_same_line(&self) -> bool {
        if !matches!(self.current(), Some(Token::Colon)) {
            return false;
        }
        // Do not treat flow contexts as candidates for nested ':' detection.
        if self.in_flow() {
            return false;
        }
        // Require that the most recent content scalar was consumed on the
        // same logical line as the current colon AND that the immediately
        // preceding token was a scalar (plain or quoted). This avoids
        // misclassifying flow punctuation or explicit-key constructs.
        let same_line = matches!(self.last_content_line_index, Some(idx) if idx == self.current_line_index);
        if !same_line {
            return false;
        }
        matches!(
            self.last_token,
            Some(Token::Plain(_)) | Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(_))
        )
    }

    /// Consume a plain scalar token
    #[allow(dead_code)]
    pub fn consume_plain_scalar(&mut self) -> Result<String, crate::error::YamlError> {
        match self.current() {
            Some(Token::Plain(s)) => {
                let result = s.clone();
                self.next()?;
                Ok(result)
            }
            Some(_token) => Err(crate::parser::document::token_errors::expected_plain_scalar(
                self.source_mut(),
            )),
            None => Err(crate::parser::document::token_errors::expected_plain_scalar_eof(
                self.source_mut(),
            )),
        }
    }

    /// Consume a quoted scalar token (single or double quoted)
    #[allow(dead_code)]
    pub fn consume_quoted_scalar(&mut self) -> Result<String, crate::error::YamlError> {
        match self.current() {
            Some(Token::SingleQuoted(s)) | Some(Token::DoubleQuoted(s)) => {
                let result = s.clone();
                self.next()?;
                Ok(result)
            }
            Some(_token) => Err(crate::parser::document::token_errors::expected_quoted_scalar(
                self.source_mut(),
            )),
            None => Err(crate::parser::document::token_errors::expected_quoted_scalar_eof(
                self.source_mut(),
            )),
        }
    }

    /// Consume any scalar token (plain, single quoted, or double quoted)
    pub fn consume_scalar(&mut self) -> Result<(String, ScalarType), crate::error::YamlError> {
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
            Some(_token) => Err(crate::parser::document::token_errors::expected_scalar(
                self.source_mut(),
            )),
            None => Err(crate::parser::document::token_errors::expected_scalar_eof(
                self.source_mut(),
            )),
        }
    }

    /// Get the current indentation level
    #[allow(dead_code)]
    pub fn indent_level(&self) -> usize {
        self.lexer.indent_level()
    }

    /// Returns the indentation level of the current line (number of leading spaces/tabs).
    pub fn line_indent(&self) -> usize {
        self.lexer.line_indent()
    }

    /// Check if the next token (after whitespace) is a colon
    #[allow(dead_code)]
    pub fn has_colon_ahead(&mut self) -> Result<bool, crate::error::YamlError> {
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

    /// Consume a single colon token, erroring if an immediate second colon follows.
    ///
    /// This enforces YAML 1.2 compliance for key-value separators in flow mappings,
    /// rejecting a double-colon sequence (::) without intervening trivia.
    /// Returns true if a colon was consumed, false if current token is not a colon.
    pub fn consume_single_colon(&mut self) -> Result<bool, crate::error::YamlError> {
        match self.current() {
            Some(Token::Colon) => {
                let _ = self.consume_if(Token::Colon)?;
                Ok(true)
            }
            _ => Err(crate::parser::document::flow_punctuation::expected_colon_in_flow_mapping(
                self.source_mut(),
            )),
        }
    }
    pub fn consume_flow_sequence_end(&mut self) -> Result<bool, crate::error::YamlError> {
        self.consume_if(Token::FlowSequenceEnd)
    }

    /// DRY helper: consume a flow mapping end ('}') if present.
    #[inline]
    pub fn consume_flow_mapping_end(&mut self) -> Result<bool, crate::error::YamlError> {
        self.consume_if(Token::FlowMappingEnd)
    }

    /// Expose a mutable reference to the underlying source for error reporting
    pub fn source_mut(&mut self) -> &mut dyn crate::io::traits::ISource {
        self.lexer.source
    }

    /// Current flow nesting depth (0 = not inside flow)
    pub fn current_flow_depth(&self) -> i32 {
        self.flow_depth
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

    #[test]
    fn test_consume_single_colon_behaviour() {
        let mut source = Buffer::new(b": value");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        // Current token should be a colon; consuming it should succeed.
        assert!(matches!(stream.current(), Some(Token::Colon)));
        let consumed = stream.consume_single_colon().unwrap();
        assert!(consumed, "Expected consume_single_colon to return true");

        // When not positioned on a colon, consume_single_colon should error.
        let mut source2 = Buffer::new(b"value");
        let mut stream2 = TokenStream::new(&mut source2, &directives, false).unwrap();
        assert!(matches!(stream2.current(), Some(Token::Plain(_))));
        let err = stream2.consume_single_colon().unwrap_err();
        assert!(err.to_string().contains(":"));
    }

    #[test]
    fn test_consume_flow_sequence_end() {
        let mut source = Buffer::new(b"[1, 2]");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        // First token is '[', second is ']'.
        assert!(matches!(stream.current(), Some(Token::FlowSequenceStart)));
        stream.next().unwrap();
        // Skip up to closing bracket.
        while !matches!(
            stream.current(),
            Some(Token::FlowSequenceEnd) | Some(Token::Eof)
        ) {
            stream.next().unwrap();
        }
        assert!(matches!(stream.current(), Some(Token::FlowSequenceEnd)));
        let consumed = stream.consume_flow_sequence_end().unwrap();
        assert!(
            consumed,
            "Expected consume_flow_sequence_end to return true when at ']' token"
        );

        // When not at a flow sequence end, helper should return false.
        let mut source2 = Buffer::new(b"[1, 2");
        let mut stream2 = TokenStream::new(&mut source2, &directives, false).unwrap();
        assert!(matches!(stream2.current(), Some(Token::FlowSequenceStart)));
        let consumed2 = stream2.consume_flow_sequence_end().unwrap();
        assert!(
            !consumed2,
            "Expected consume_flow_sequence_end to return false when not at ']' token"
        );
    }

    #[test]
    fn test_consume_flow_mapping_end() {
        let mut source = Buffer::new(b"{ key: value }");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        // First token is '{', eventually followed by '}'.
        assert!(matches!(stream.current(), Some(Token::FlowMappingStart)));
        stream.next().unwrap();
        while !matches!(
            stream.current(),
            Some(Token::FlowMappingEnd) | Some(Token::Eof)
        ) {
            stream.next().unwrap();
        }
        assert!(matches!(stream.current(), Some(Token::FlowMappingEnd)));
        let consumed = stream.consume_flow_mapping_end().unwrap();
        assert!(
            consumed,
            "Expected consume_flow_mapping_end to return true when at closing brace token"
        );

        // When not at a flow mapping end, helper should return false.
        let mut source2 = Buffer::new(b"{");
        let mut stream2 = TokenStream::new(&mut source2, &directives, false).unwrap();
        assert!(matches!(stream2.current(), Some(Token::FlowMappingStart)));
        let consumed2 = stream2.consume_flow_mapping_end().unwrap();
        assert!(
            !consumed2,
            "Expected consume_flow_mapping_end to return false when not at closing brace token"
        );
    }
}
