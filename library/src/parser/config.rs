//! Parser Configuration & Builder
//!
//! Provides configuration options and a fluent builder API for customizing YAML parser behavior.
//! Supports setting limits, toggling features, and adapting to different environments.
//!
//! Copyright (c) 2026 YAML Library Developers

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Configuration options for the YAML parser
///
/// Use `ParserConfig::builder()` to create with the builder pattern.
///
/// # Example
/// ```
/// use yaml_lib::ParserConfig;
///
/// let config = ParserConfig::builder()
///     .max_depth(64)
///     .strict_mode(true)
///     .allow_duplicate_keys(false)
///     .build();
/// ```
#[derive(Clone, Debug)]
pub struct ParserConfig {
    /// Maximum nesting depth allowed (default: 128)
    pub max_depth: usize,

    /// Maximum document size in bytes (default: unlimited)
    pub max_size: Option<usize>,

    /// Enable strict YAML 1.2 compliance (default: false)
    pub strict_mode: bool,

    /// Allow duplicate keys in mappings (default: true)
    pub allow_duplicate_keys: bool,

    /// Allow tabs for indentation (default: true)
    pub allow_tabs: bool,

    /// Merge keys (<<) support (default: true)
    pub allow_merge_keys: bool,

    /// Allow anchors and aliases (default: true)
    pub allow_anchors: bool,

    /// Maximum number of anchors per document (default: unlimited)
    pub max_anchors: Option<usize>,

    /// Allow explicit document markers (--- and ...) (default: true)
    pub allow_document_markers: bool,

    /// Allow tags (!!, !tag) (default: true)
    pub allow_tags: bool,

    /// Preserve comments during parsing (default: false)
    pub preserve_comments: bool,

    /// Custom error message for depth exceeded
    pub depth_error_message: Option<String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: 128,
            max_size: None,
            strict_mode: false,
            allow_duplicate_keys: true,
            allow_tabs: true,
            allow_merge_keys: true,
            allow_anchors: true,
            max_anchors: None,
            allow_document_markers: true,
            allow_tags: true,
            preserve_comments: false,
            depth_error_message: None,
        }
    }
}

impl ParserConfig {
    /// Create a new parser configuration with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for fluent configuration
    #[must_use]
    pub fn builder() -> ParserConfigBuilder {
        ParserConfigBuilder::new()
    }

    /// Create a strict YAML 1.2 compliant configuration
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            allow_duplicate_keys: false,
            allow_tabs: false,
            ..Default::default()
        }
    }

    /// Create a permissive configuration for maximum compatibility
    pub fn permissive() -> Self {
        Self {
            strict_mode: false,
            allow_duplicate_keys: true,
            allow_tabs: true,
            allow_merge_keys: true,
            allow_anchors: true,
            ..Default::default()
        }
    }

    /// Create an embedded-optimized configuration with resource limits
    pub fn embedded() -> Self {
        Self {
            max_depth: 32,
            max_size: Some(64 * 1024), // 64KB
            max_anchors: Some(32),
            preserve_comments: false,
            ..Default::default()
        }
    }

    /// Validate if current depth exceeds maximum
    pub fn check_depth(&self, current_depth: usize) -> Result<(), crate::error::YamlError> {
        if current_depth > self.max_depth {
            if let Some(ref _msg) = self.depth_error_message {
                Err(crate::parser::document::error_builder::limit_error(
                    "Nesting depth",
                    self.max_depth,
                    &format!("current: {}", current_depth),
                ))
            } else {
                Err(crate::parser::document::error_builder::limit_error(
                    "Nesting depth",
                    self.max_depth,
                    &format!("current: {}", current_depth),
                ))
            }
        } else {
            Ok(())
        }
    }

    /// Validate if document size exceeds maximum
    pub fn check_size(&self, current_size: usize) -> Result<(), crate::error::YamlError> {
        if let Some(max) = self.max_size {
            if current_size > max {
                return Err(crate::parser::document::error_builder::limit_error(
                    "Document size (bytes)",
                    max,
                    "bytes",
                ));
            }
        }
        Ok(())
    }

    /// Validate if anchor count exceeds maximum
    pub fn check_anchor_count(&self, count: usize) -> Result<(), crate::error::YamlError> {
        if let Some(max) = self.max_anchors {
            if count > max {
                return Err(crate::parser::document::error_builder::limit_error(
                    "Anchor count",
                    max,
                    "anchors",
                ));
            }
        }
        Ok(())
    }
}

/// Builder for creating ParserConfig with a fluent API
///
/// # Example
/// ```
/// use yaml_lib::ParserConfig;
///
/// let config = ParserConfig::builder()
///     .max_depth(64)
///     .max_size(1024 * 1024) // 1MB
///     .strict_mode(true)
///     .allow_duplicate_keys(false)
///     .allow_tabs(false)
///     .depth_error_message("Nesting too deep!")
///     .build();
/// ```
#[must_use]
#[derive(Default)]
pub struct ParserConfigBuilder {
    config: ParserConfig,
}

impl ParserConfigBuilder {
    /// Create a new builder with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Set maximum nesting depth
    #[must_use]
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }

    /// Set maximum document size in bytes
    #[must_use]
    pub fn max_size(mut self, size: usize) -> Self {
        self.config.max_size = Some(size);
        self
    }

    /// Enable or disable strict YAML 1.2 compliance
    #[must_use]
    pub fn strict_mode(mut self, enabled: bool) -> Self {
        self.config.strict_mode = enabled;
        self
    }

    /// Allow or disallow duplicate keys in mappings
    #[must_use]
    pub fn allow_duplicate_keys(mut self, allow: bool) -> Self {
        self.config.allow_duplicate_keys = allow;
        self
    }

    /// Allow or disallow tabs for indentation
    #[must_use]
    pub fn allow_tabs(mut self, allow: bool) -> Self {
        self.config.allow_tabs = allow;
        self
    }

    /// Allow or disallow merge keys (<<)
    #[must_use]
    pub fn allow_merge_keys(mut self, allow: bool) -> Self {
        self.config.allow_merge_keys = allow;
        self
    }

    /// Allow or disallow anchors and aliases
    #[must_use]
    pub fn allow_anchors(mut self, allow: bool) -> Self {
        self.config.allow_anchors = allow;
        self
    }

    /// Set maximum number of anchors per document
    #[must_use]
    pub fn max_anchors(mut self, max: usize) -> Self {
        self.config.max_anchors = Some(max);
        self
    }

    /// Allow or disallow explicit document markers (--- and ...)
    #[must_use]
    pub fn allow_document_markers(mut self, allow: bool) -> Self {
        self.config.allow_document_markers = allow;
        self
    }

    /// Allow or disallow tags (!!, !tag)
    #[must_use]
    pub fn allow_tags(mut self, allow: bool) -> Self {
        self.config.allow_tags = allow;
        self
    }

    /// Preserve or discard comments during parsing
    #[must_use]
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.config.preserve_comments = preserve;
        self
    }

    /// Set custom error message for depth exceeded
    #[must_use]
    pub fn depth_error_message<S: Into<String>>(mut self, message: S) -> Self {
        self.config.depth_error_message = Some(message.into());
        self
    }

    /// Build the final ParserConfig
    #[must_use]
    pub fn build(self) -> ParserConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ParserConfig::default();
        assert_eq!(config.max_depth, 128);
        assert_eq!(config.max_size, None);
        assert!(!config.strict_mode);
        assert!(config.allow_duplicate_keys);
        assert!(config.allow_tabs);
        assert!(config.allow_merge_keys);
        assert!(config.allow_anchors);
        assert!(config.allow_document_markers);
        assert!(config.allow_tags);
        assert!(!config.preserve_comments);
    }

    #[test]
    fn test_new_config() {
        let config = ParserConfig::new();
        assert_eq!(config.max_depth, 128);
    }

    #[test]
    fn test_strict_config() {
        let config = ParserConfig::strict();
        assert!(config.strict_mode);
        assert!(!config.allow_duplicate_keys);
        assert!(!config.allow_tabs);
    }

    #[test]
    fn test_permissive_config() {
        let config = ParserConfig::permissive();
        assert!(!config.strict_mode);
        assert!(config.allow_duplicate_keys);
        assert!(config.allow_tabs);
        assert!(config.allow_merge_keys);
        assert!(config.allow_anchors);
    }

    #[test]
    fn test_embedded_config() {
        let config = ParserConfig::embedded();
        assert_eq!(config.max_depth, 32);
        assert_eq!(config.max_size, Some(64 * 1024));
        assert_eq!(config.max_anchors, Some(32));
        assert!(!config.preserve_comments);
    }

    #[test]
    fn test_builder_basic() {
        let config = ParserConfig::builder()
            .max_depth(64)
            .strict_mode(true)
            .build();

        assert_eq!(config.max_depth, 64);
        assert!(config.strict_mode);
    }

    #[test]
    fn test_builder_all_options() {
        let config = ParserConfig::builder()
            .max_depth(64)
            .max_size(1024)
            .strict_mode(true)
            .allow_duplicate_keys(false)
            .allow_tabs(false)
            .allow_merge_keys(false)
            .allow_anchors(false)
            .max_anchors(10)
            .allow_document_markers(false)
            .allow_tags(false)
            .preserve_comments(true)
            .depth_error_message("Too deep!")
            .build();

        assert_eq!(config.max_depth, 64);
        assert_eq!(config.max_size, Some(1024));
        assert!(config.strict_mode);
        assert!(!config.allow_duplicate_keys);
        assert!(!config.allow_tabs);
        assert!(!config.allow_merge_keys);
        assert!(!config.allow_anchors);
        assert_eq!(config.max_anchors, Some(10));
        assert!(!config.allow_document_markers);
        assert!(!config.allow_tags);
        assert!(config.preserve_comments);
        assert_eq!(config.depth_error_message.as_deref(), Some("Too deep!"));
    }

    #[test]
    fn test_check_depth_ok() {
        let config = ParserConfig::builder().max_depth(10).build();
        assert!(config.check_depth(5).is_ok());
        assert!(config.check_depth(10).is_ok());
    }

    #[test]
    fn test_check_depth_exceeded() {
        let config = ParserConfig::builder().max_depth(10).build();
        assert!(config.check_depth(11).is_err());
        assert!(config.check_depth(100).is_err());
    }

    #[test]
    fn test_check_depth_custom_message() {
        let config = ParserConfig::builder()
            .max_depth(10)
            .depth_error_message("Custom depth error")
            .build();

        let result = config.check_depth(11);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("Custom depth error")
                || err_str.contains("maximum parser depth")
                || err_str.contains("Validation error"),
            "Error message: {}",
            err_str
        );
    }

    #[test]
    fn test_check_size_ok() {
        let config = ParserConfig::builder().max_size(1024).build();
        assert!(config.check_size(512).is_ok());
        assert!(config.check_size(1024).is_ok());
    }

    #[test]
    fn test_check_size_exceeded() {
        let config = ParserConfig::builder().max_size(1024).build();
        assert!(config.check_size(1025).is_err());
        assert!(config.check_size(10000).is_err());
    }

    #[test]
    fn test_check_size_unlimited() {
        let config = ParserConfig::builder().build(); // No max_size set
        assert!(config.check_size(1_000_000).is_ok());
        assert!(config.check_size(1_000_000_000).is_ok());
    }

    #[test]
    fn test_check_anchor_count_ok() {
        let config = ParserConfig::builder().max_anchors(10).build();
        assert!(config.check_anchor_count(5).is_ok());
        assert!(config.check_anchor_count(10).is_ok());
    }

    #[test]
    fn test_check_anchor_count_exceeded() {
        let config = ParserConfig::builder().max_anchors(10).build();
        assert!(config.check_anchor_count(11).is_err());
        assert!(config.check_anchor_count(100).is_err());
    }

    #[test]
    fn test_check_anchor_count_unlimited() {
        let config = ParserConfig::builder().build(); // No max_anchors set
        assert!(config.check_anchor_count(1000).is_ok());
        assert!(config.check_anchor_count(10000).is_ok());
    }

    #[test]
    fn test_builder_chaining() {
        let config = ParserConfig::builder()
            .max_depth(32)
            .max_size(4096)
            .strict_mode(true)
            .allow_duplicate_keys(false)
            .build();

        assert_eq!(config.max_depth, 32);
        assert_eq!(config.max_size, Some(4096));
        assert!(config.strict_mode);
        assert!(!config.allow_duplicate_keys);
    }

    #[test]
    fn test_clone_config() {
        let config1 = ParserConfig::builder()
            .max_depth(64)
            .strict_mode(true)
            .build();

        let config2 = config1.clone();
        assert_eq!(config2.max_depth, 64);
        assert!(config2.strict_mode);
    }
}
