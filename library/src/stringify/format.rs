//! Formatting options and control for YAML serialization
//!
//! Provides fine-grained control over YAML output formatting including
//! indentation, line width, quote styles, and collection formatting.

use alloc::string::String;

use crate::io::destinations::buffer::Buffer as BufferDestination;
use crate::nodes::node::{Node, Numeric};
use crate::stringify::default::stringify as yaml_stringify;

/// YAML output formatting options
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Number of spaces per indentation level (default: 2)
    pub indent: usize,
    /// Maximum line width before wrapping (default: 80, 0 = no limit)
    pub line_width: usize,
    /// Preferred quote style for strings
    pub quote_style: QuoteStyle,
    /// How to format collections
    pub collection_style: CollectionStyle,
    /// Whether to emit document start marker (---)
    pub explicit_start: bool,
    /// Whether to emit document end marker (...)
    pub explicit_end: bool,
    /// Number of newlines between documents (default: 1)
    pub document_separator_lines: usize,
    /// Whether to preserve original formatting hints
    pub preserve_formatting: bool,
    /// Whether to sort mapping keys
    pub sort_keys: bool,
    /// Whether to emit null values
    pub emit_null: bool,
    /// Whether to use flow style for empty collections
    pub flow_empty_collections: bool,
    /// Minimum collection size to use block style (default: 3)
    pub block_threshold: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: 2,
            line_width: 80,
            quote_style: QuoteStyle::Auto,
            collection_style: CollectionStyle::Auto,
            explicit_start: false,
            explicit_end: false,
            document_separator_lines: 1,
            preserve_formatting: true,
            sort_keys: false,
            emit_null: true,
            flow_empty_collections: true,
            block_threshold: 3,
        }
    }
}

impl FormatOptions {
    /// Create new default format options
    pub fn new() -> Self {
        Self::default()
    }

    /// Compact formatting (minimal whitespace)
    pub fn compact() -> Self {
        Self {
            indent: 2,
            line_width: 0,
            quote_style: QuoteStyle::Auto,
            collection_style: CollectionStyle::Flow,
            explicit_start: false,
            explicit_end: false,
            document_separator_lines: 0,
            preserve_formatting: false,
            sort_keys: false,
            emit_null: false,
            flow_empty_collections: true,
            block_threshold: 100,
        }
    }

    /// Pretty formatting (readable, well-spaced)
    pub fn pretty() -> Self {
        Self {
            indent: 2,
            line_width: 80,
            quote_style: QuoteStyle::Auto,
            collection_style: CollectionStyle::Block,
            explicit_start: true,
            explicit_end: false,
            document_separator_lines: 1,
            preserve_formatting: false,
            sort_keys: true,
            emit_null: true,
            flow_empty_collections: true,
            block_threshold: 3,
        }
    }

    /// Minimal formatting (bare minimum valid YAML)
    pub fn minimal() -> Self {
        Self {
            indent: 2,
            line_width: 0,
            quote_style: QuoteStyle::None,
            collection_style: CollectionStyle::Flow,
            explicit_start: false,
            explicit_end: false,
            document_separator_lines: 0,
            preserve_formatting: false,
            sort_keys: false,
            emit_null: false,
            flow_empty_collections: true,
            block_threshold: 100,
        }
    }

    /// Builder method: set indentation
    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Builder method: set line width
    pub fn with_line_width(mut self, width: usize) -> Self {
        self.line_width = width;
        self
    }

    /// Builder method: set quote style
    pub fn with_quote_style(mut self, style: QuoteStyle) -> Self {
        self.quote_style = style;
        self
    }

    /// Builder method: set collection style
    pub fn with_collection_style(mut self, style: CollectionStyle) -> Self {
        self.collection_style = style;
        self
    }

    /// Builder method: enable explicit document markers
    pub fn with_explicit_markers(mut self, start: bool, end: bool) -> Self {
        self.explicit_start = start;
        self.explicit_end = end;
        self
    }

    /// Builder method: enable key sorting
    pub fn with_sorted_keys(mut self, sort: bool) -> Self {
        self.sort_keys = sort;
        self
    }
}

/// Quote style for string values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    /// Choose automatically based on content
    Auto,
    /// Prefer no quotes when possible
    None,
    /// Prefer single quotes
    Single,
    /// Prefer double quotes
    Double,
    /// Always use single quotes
    AlwaysSingle,
    /// Always use double quotes
    AlwaysDouble,
}

/// Collection formatting style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionStyle {
    /// Choose automatically based on size and content
    Auto,
    /// Prefer block style (multi-line)
    Block,
    /// Prefer flow style (inline)
    Flow,
    /// Force block style
    AlwaysBlock,
    /// Force flow style
    AlwaysFlow,
}

/// Formatting context for recursive serialization
#[derive(Debug, Clone)]
pub struct FormatContext {
    /// Current indentation level
    pub level: usize,
    /// Current column position
    pub column: usize,
    /// Parent collection style
    pub parent_style: CollectionStyle,
    /// Whether we're at the start of a line
    pub at_line_start: bool,
}

impl FormatContext {
    /// Create new format context
    pub fn new() -> Self {
        Self {
            level: 0,
            column: 0,
            parent_style: CollectionStyle::Block,
            at_line_start: true,
        }
    }

    /// Increase indentation level
    pub fn indent(&mut self) {
        self.level += 1;
    }

    /// Decrease indentation level
    pub fn dedent(&mut self) {
        if self.level > 0 {
            self.level -= 1;
        }
    }

    /// Get indentation string
    pub fn indent_str(&self, options: &FormatOptions) -> String {
        " ".repeat(self.level * options.indent)
    }

    /// Update column position
    pub fn advance(&mut self, count: usize) {
        self.column += count;
        self.at_line_start = false;
    }

    /// Reset to new line
    pub fn newline(&mut self) {
        self.column = 0;
        self.at_line_start = true;
    }
}

impl Default for FormatContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a `Numeric` value to a human-readable string.
///
/// This is a thin wrapper around `Numeric::to_string_lossy` so that
/// callers outside of `nodes` can depend on a stable helper in the
/// stringify layer.
pub fn numeric_to_string_lossy(num: &Numeric) -> String {
    num.to_string_lossy()
}

/// Convert a `Node` into a general-purpose string representation.
///
/// This is intended for diagnostics, devtools, and non-critical
/// formatting paths. Complex structures fall back to YAML stringify;
/// if that fails, a `Debug` representation is used as a last resort.
pub fn node_to_string_lossy(node: &Node) -> String {
    match node {
        Node::Str(s, _, _) => s.clone(),
        Node::Number(num) => numeric_to_string_lossy(num),
        Node::Boolean(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Node::None => "null".to_string(),
        Node::Comment(c) => c.clone(),
        Node::Alias(name) => name.clone(),
        // For all structural/container forms, fall back to YAML
        // stringify to get a stable textual view.
        _ => {
            let mut buf = BufferDestination::new();
            if yaml_stringify(node, &mut buf).is_ok() {
                buf.to_string()
            } else {
                format!("{:?}", node)
            }
        }
    }
}

/// Convert a `Node` into a string suitable for use as a mapping key.
///
/// Key-specific tweaks over `node_to_string_lossy`:
/// - `None` is rendered as an empty string.
/// - Other types reuse the lossy representation.
pub fn node_to_key_like_string(node: &Node) -> String {
    match node {
        Node::None => String::new(),
        _ => node_to_string_lossy(node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = FormatOptions::default();
        assert_eq!(opts.indent, 2);
        assert_eq!(opts.line_width, 80);
        assert!(!opts.explicit_start);
        assert!(opts.preserve_formatting);
    }

    #[test]
    fn test_compact_options() {
        let opts = FormatOptions::compact();
        assert_eq!(opts.line_width, 0);
        assert_eq!(opts.collection_style, CollectionStyle::Flow);
        assert!(!opts.emit_null);
        assert!(!opts.preserve_formatting);
    }

    #[test]
    fn test_pretty_options() {
        let opts = FormatOptions::pretty();
        assert!(opts.explicit_start);
        assert!(opts.sort_keys);
        assert_eq!(opts.collection_style, CollectionStyle::Block);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = FormatOptions::new()
            .with_indent(4)
            .with_line_width(120)
            .with_sorted_keys(true)
            .with_explicit_markers(true, true);

        assert_eq!(opts.indent, 4);
        assert_eq!(opts.line_width, 120);
        assert!(opts.sort_keys);
        assert!(opts.explicit_start);
        assert!(opts.explicit_end);
    }

    #[test]
    fn test_format_context() {
        let mut ctx = FormatContext::new();
        assert_eq!(ctx.level, 0);
        assert!(ctx.at_line_start);

        ctx.indent();
        assert_eq!(ctx.level, 1);

        ctx.advance(5);
        assert_eq!(ctx.column, 5);
        assert!(!ctx.at_line_start);

        ctx.newline();
        assert_eq!(ctx.column, 0);
        assert!(ctx.at_line_start);

        ctx.dedent();
        assert_eq!(ctx.level, 0);
    }

    #[test]
    fn test_indent_str() {
        let opts = FormatOptions::new();
        let mut ctx = FormatContext::new();

        assert_eq!(ctx.indent_str(&opts), "");

        ctx.indent();
        assert_eq!(ctx.indent_str(&opts), "  ");

        ctx.indent();
        assert_eq!(ctx.indent_str(&opts), "    ");
    }

    #[test]
    fn test_indent_str_custom() {
        let opts = FormatOptions::new().with_indent(4);
        let mut ctx = FormatContext::new();

        ctx.indent();
        assert_eq!(ctx.indent_str(&opts), "    ");

        ctx.indent();
        assert_eq!(ctx.indent_str(&opts), "        ");
    }
}
