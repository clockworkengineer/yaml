  
//! YAML Error Recovery Strategies
//!
//! Provides error recovery mechanisms for YAML parsing, allowing parsing to continue
//! after encountering errors and collecting multiple errors for robust diagnostics.
//! Includes error collection, recovery strategies, and integration with enhanced error handling.
//!
//! Copyright (c) 2026 YAML Library Developers

use alloc::vec::Vec;

use crate::error::enhanced::EnhancedError;
use crate::error::{ErrorKind, YamlError};

// Re-export RecoveryStrategy from enhanced module
pub use crate::error::enhanced::RecoveryStrategy;

/// Collection of errors encountered during parsing with recovery
#[derive(Debug)]
pub struct ErrorCollection {
    /// Collected errors
    errors: Vec<EnhancedError>,
    /// Whether to continue after errors
    continue_on_error: bool,
    /// Maximum number of errors to collect before aborting
    max_errors: usize,
}

impl ErrorCollection {
    /// Create a new error collection
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            continue_on_error: true,
            max_errors: 100,
        }
    }

    /// Create with specific configuration
    pub fn with_config(continue_on_error: bool, max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            continue_on_error,
            max_errors,
        }
    }

    /// Add an error to the collection
    pub fn add(&mut self, error: EnhancedError) {
        self.errors.push(error);
    }

    /// Check if we should continue parsing
    pub fn should_continue(&self) -> bool {
        self.continue_on_error && self.errors.len() < self.max_errors
    }

    /// Get all collected errors
    pub fn errors(&self) -> &[EnhancedError] {
        &self.errors
    }

    /// Get number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }

    /// Convert to a single error (using first error if any)
    pub fn into_result<T>(self, value: T) -> Result<T, EnhancedError> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self.errors.into_iter().next().unwrap())
        }
    }
}

impl Default for ErrorCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Error recovery handler
pub struct RecoveryHandler {
    /// Strategy to use for different error kinds
    strategies: [RecoveryStrategy; 13],
}

impl RecoveryHandler {
    /// Create a new recovery handler with default strategies
    pub fn new() -> Self {
        Self {
            strategies: [
                RecoveryStrategy::SkipLine,      // SyntaxError
                RecoveryStrategy::SkipLine,      // ParseError
                RecoveryStrategy::Abort,         // UnterminatedString
                RecoveryStrategy::SkipLine,      // InvalidTag
                RecoveryStrategy::InsertDefault, // UndefinedAlias
                RecoveryStrategy::SkipLine,      // InvalidAnchor
                RecoveryStrategy::SkipLine,      // DuplicateAnchor
                RecoveryStrategy::Abort,         // IoError
                RecoveryStrategy::SkipLine,      // ValidationError
                RecoveryStrategy::Abort,         // UnexpectedEof
                RecoveryStrategy::SkipLine,      // UnexpectedCharacter
                RecoveryStrategy::SkipLine,      // InvalidEscape
                RecoveryStrategy::Abort,         // Unsupported
            ],
        }
    }

    /// Create a recovery handler that never recovers (strict mode)
    pub fn strict() -> Self {
        Self {
            strategies: [RecoveryStrategy::Abort; 13],
        }
    }

    /// Create a recovery handler that always tries to recover
    pub fn lenient() -> Self {
        Self {
            strategies: [
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::InsertDefault,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
                RecoveryStrategy::SkipLine,
            ],
        }
    }

    /// Set strategy for a specific error kind
    pub fn set_strategy(&mut self, kind: &ErrorKind, strategy: RecoveryStrategy) {
        let index = error_kind_to_index(kind);
        self.strategies[index] = strategy;
    }

    /// Get strategy for a specific error kind
    pub fn get_strategy(&self, kind: &ErrorKind) -> RecoveryStrategy {
        let index = error_kind_to_index(kind);
        self.strategies[index]
    }

    /// Attempt to recover from an error
    pub fn recover(&self, error: &YamlError) -> RecoveryStrategy {
        self.get_strategy(error.kind())
    }
}

impl Default for RecoveryHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert ErrorKind to array index
fn error_kind_to_index(kind: &ErrorKind) -> usize {
    match kind {
        ErrorKind::SyntaxError => 0,
        ErrorKind::ParseError => 1,
        ErrorKind::UnterminatedString => 2,
        ErrorKind::InvalidTag => 3,
        ErrorKind::UndefinedAlias => 4,
        ErrorKind::InvalidAnchor => 5,
        ErrorKind::DuplicateAnchor => 6,
        ErrorKind::IoError => 7,
        ErrorKind::ValidationError => 8,
        ErrorKind::UnexpectedEof => 9,
        ErrorKind::UnexpectedCharacter => 10,
        ErrorKind::InvalidEscape => 11,
        ErrorKind::Unsupported => 12,
    }
}

/// Parser state for recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// At document start
    DocumentStart,
    /// In block mapping context
    BlockMapping,
    /// In block sequence context
    BlockSequence,
    /// In flow mapping context
    FlowMapping,
    /// In flow sequence context
    FlowSequence,
    /// In scalar value
    Scalar,
    /// At document end
    DocumentEnd,
}

/// Recovery context providing state information
#[derive(Debug)]
pub struct RecoveryContext {
    /// Current parser state
    pub state: ParserState,
    /// Current indentation level
    pub indent_level: usize,
    /// Whether we're in a flow context
    pub in_flow: bool,
    /// Number of open brackets/braces
    pub bracket_depth: usize,
}

impl RecoveryContext {
    /// Create a new recovery context
    pub fn new() -> Self {
        Self {
            state: ParserState::DocumentStart,
            indent_level: 0,
            in_flow: false,
            bracket_depth: 0,
        }
    }

    /// Update state for block mapping
    pub fn enter_block_mapping(&mut self) {
        self.state = ParserState::BlockMapping;
    }

    /// Update state for flow mapping
    pub fn enter_flow_mapping(&mut self) {
        self.state = ParserState::FlowMapping;
        self.in_flow = true;
        self.bracket_depth += 1;
    }

    /// Exit flow context
    pub fn exit_flow(&mut self) {
        if self.bracket_depth > 0 {
            self.bracket_depth -= 1;
        }
        if self.bracket_depth == 0 {
            self.in_flow = false;
        }
    }

    /// Check if recovery is safe at current position
    pub fn can_recover(&self, strategy: RecoveryStrategy) -> bool {
        match strategy {
            RecoveryStrategy::Abort => false,
            RecoveryStrategy::SkipLine => true,
            RecoveryStrategy::SkipToNextMapping => {
                matches!(
                    self.state,
                    ParserState::BlockMapping | ParserState::FlowMapping
                )
            }
            RecoveryStrategy::SkipCollection => self.bracket_depth > 0 || self.indent_level > 0,
            RecoveryStrategy::InsertDefault => true,
        }
    }
}

impl Default for RecoveryContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_collection() {
        let mut collection = ErrorCollection::new();
        assert!(collection.is_empty());
        assert!(collection.should_continue());

        let error = EnhancedError::new(YamlError::new(ErrorKind::SyntaxError, "test"));
        collection.add(error);

        assert_eq!(collection.len(), 1);
        assert!(!collection.is_empty());
    }

    #[test]
    fn test_error_collection_max() {
        let mut collection = ErrorCollection::with_config(true, 2);

        collection.add(EnhancedError::new(YamlError::new(
            ErrorKind::SyntaxError,
            "error1",
        )));
        assert!(collection.should_continue());

        collection.add(EnhancedError::new(YamlError::new(
            ErrorKind::SyntaxError,
            "error2",
        )));
        assert!(!collection.should_continue());
    }

    #[test]
    fn test_recovery_handler_default() {
        let handler = RecoveryHandler::new();

        let error = YamlError::new(ErrorKind::SyntaxError, "test");
        assert_eq!(handler.recover(&error), RecoveryStrategy::SkipLine);

        let error = YamlError::new(ErrorKind::UnexpectedEof, "test");
        assert_eq!(handler.recover(&error), RecoveryStrategy::Abort);
    }

    #[test]
    fn test_recovery_handler_strict() {
        let handler = RecoveryHandler::strict();

        let error = YamlError::new(ErrorKind::SyntaxError, "test");
        assert_eq!(handler.recover(&error), RecoveryStrategy::Abort);
    }

    #[test]
    fn test_recovery_handler_lenient() {
        let handler = RecoveryHandler::lenient();

        let error = YamlError::new(ErrorKind::UnexpectedEof, "test");
        assert_eq!(handler.recover(&error), RecoveryStrategy::SkipLine);
    }

    #[test]
    fn test_recovery_handler_custom() {
        let mut handler = RecoveryHandler::new();
        handler.set_strategy(&ErrorKind::SyntaxError, RecoveryStrategy::Abort);

        let error = YamlError::new(ErrorKind::SyntaxError, "test");
        assert_eq!(handler.recover(&error), RecoveryStrategy::Abort);
    }

    #[test]
    fn test_recovery_context() {
        let mut ctx = RecoveryContext::new();
        assert_eq!(ctx.state, ParserState::DocumentStart);
        assert!(!ctx.in_flow);

        ctx.enter_flow_mapping();
        assert_eq!(ctx.state, ParserState::FlowMapping);
        assert!(ctx.in_flow);
        assert_eq!(ctx.bracket_depth, 1);

        ctx.exit_flow();
        assert!(!ctx.in_flow);
        assert_eq!(ctx.bracket_depth, 0);
    }

    #[test]
    fn test_recovery_context_can_recover() {
        let mut ctx = RecoveryContext::new();

        assert!(ctx.can_recover(RecoveryStrategy::SkipLine));
        assert!(!ctx.can_recover(RecoveryStrategy::Abort));

        ctx.enter_block_mapping();
        assert!(ctx.can_recover(RecoveryStrategy::SkipToNextMapping));
    }
      #[test]
    fn test_error_collection_clear() {
        let mut collection = ErrorCollection::new();
        collection.add(EnhancedError::new(YamlError::new(ErrorKind::SyntaxError, "test")));
        assert!(!collection.is_empty());
        collection.clear();
        assert!(collection.is_empty());
    }

    #[test]
    fn test_error_collection_into_result() {
        let collection = ErrorCollection::new();
        let result: Result<i32, EnhancedError> = collection.into_result(42);
        assert_eq!(result.unwrap(), 42);

        let mut collection = ErrorCollection::new();
        collection.add(EnhancedError::new(YamlError::new(ErrorKind::SyntaxError, "fail")));
        let result: Result<i32, EnhancedError> = collection.into_result(42);
        assert!(result.is_err());
    }

    #[test]
    fn test_recovery_handler_strict_abort() {
        let handler = RecoveryHandler::strict();
        let error = YamlError::new(ErrorKind::ParseError, "test");
        assert_eq!(handler.recover(&error), RecoveryStrategy::Abort);
    }

    #[test]
    fn test_recovery_context_deep_bracket() {
        let mut ctx = RecoveryContext::new();
        for _ in 0..5 {
            ctx.enter_flow_mapping();
        }
        assert_eq!(ctx.bracket_depth, 5);
        assert!(ctx.in_flow);
        ctx.exit_flow();
        assert_eq!(ctx.bracket_depth, 4);
        ctx.exit_flow();
        ctx.exit_flow();
        ctx.exit_flow();
        ctx.exit_flow();
        assert_eq!(ctx.bracket_depth, 0);
        assert!(!ctx.in_flow);
    }
}
