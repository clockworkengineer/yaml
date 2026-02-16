//! Performance optimization utilities for YAML parsing and processing
//!
//! This module provides various optimizations including:
//! - Lazy tag coercion (defer type conversion until needed)
//! - Capacity hints for pre-allocation
//! - Zero-copy string operations where possible
//! - Performance profiling helpers

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::Node;

/// Lazy tag that defers type coercion until accessed
///
/// This avoids unnecessary conversions when tags are applied but the value
/// is never used or accessed in its converted form.
#[derive(Debug, Clone)]
pub struct LazyTag {
    /// The original string value before coercion
    pub raw_value: String,
    /// The tag name (e.g., "!!int", "!!float", "!!bool")
    pub tag: String,
    /// Cached coerced value (computed on first access)
    cached: Option<Node>,
}

impl LazyTag {
    /// Create a new lazy tag
    pub fn new(raw_value: String, tag: String) -> Self {
        Self {
            raw_value,
            tag,
            cached: None,
        }
    }

    /// Get the coerced value, computing it if necessary
    pub fn get_or_coerce(&mut self) -> &Node {
        if self.cached.is_none() {
            self.cached = Some(self.coerce());
        }
        self.cached.as_ref().unwrap()
    }

    /// Check if the value has been coerced yet
    pub fn is_coerced(&self) -> bool {
        self.cached.is_some()
    }

    /// Force coercion immediately
    fn coerce(&self) -> Node {
        // Simplified coercion - actual implementation would be more comprehensive
        match self.tag.as_str() {
            "!!int" | "tag:yaml.org,2002:int" => {
                if let Ok(i) = self.raw_value.parse::<i64>() {
                    Node::Number(crate::Numeric::Integer(i))
                } else {
                    Node::Str(
                        self.raw_value.clone(),
                        crate::QuoteType::Unquoted,
                        crate::BlockStyle::None,
                    )
                }
            }
            "!!float" | "tag:yaml.org,2002:float" => {
                if let Ok(f) = self.raw_value.parse::<f64>() {
                    Node::Number(crate::Numeric::Float(f))
                } else {
                    Node::Str(
                        self.raw_value.clone(),
                        crate::QuoteType::Unquoted,
                        crate::BlockStyle::None,
                    )
                }
            }
            "!!bool" | "tag:yaml.org,2002:bool" => match self.raw_value.to_lowercase().as_str() {
                "true" | "yes" | "on" => Node::Boolean(true),
                "false" | "no" | "off" => Node::Boolean(false),
                _ => Node::Str(
                    self.raw_value.clone(),
                    crate::QuoteType::Unquoted,
                    crate::BlockStyle::None,
                ),
            },
            "!!null" | "tag:yaml.org,2002:null" => Node::None,
            "!!str" | "tag:yaml.org,2002:str" => Node::Str(
                self.raw_value.clone(),
                crate::QuoteType::Unquoted,
                crate::BlockStyle::None,
            ),
            _ => {
                // Unknown tag - preserve as string
                Node::Str(
                    self.raw_value.clone(),
                    crate::QuoteType::Unquoted,
                    crate::BlockStyle::None,
                )
            }
        }
    }
}

/// Capacity hints for optimizing allocations during parsing
///
/// These hints help pre-allocate the right amount of memory to avoid
/// repeated reallocations during document construction.
#[derive(Debug, Clone, Copy)]
pub struct CapacityHints {
    /// Expected number of mapping pairs at current level
    pub mapping_pairs: usize,
    /// Expected number of sequence items at current level
    pub sequence_items: usize,
    /// Expected string length for keys/values
    pub string_capacity: usize,
    /// Expected nesting depth
    pub nesting_depth: usize,
}

impl CapacityHints {
    /// Create default capacity hints
    pub fn new() -> Self {
        Self {
            mapping_pairs: 8,
            sequence_items: 8,
            string_capacity: 32,
            nesting_depth: 4,
        }
    }

    /// Create capacity hints optimized for small documents
    pub fn small() -> Self {
        Self {
            mapping_pairs: 4,
            sequence_items: 4,
            string_capacity: 16,
            nesting_depth: 2,
        }
    }

    /// Create capacity hints optimized for large documents
    pub fn large() -> Self {
        Self {
            mapping_pairs: 32,
            sequence_items: 32,
            string_capacity: 64,
            nesting_depth: 8,
        }
    }

    /// Create capacity hints from document statistics
    pub fn from_stats(node_count: usize, max_depth: usize) -> Self {
        let avg_size = (node_count / max_depth.max(1)).max(4);
        Self {
            mapping_pairs: avg_size,
            sequence_items: avg_size,
            string_capacity: 32,
            nesting_depth: max_depth,
        }
    }

    /// Update hints based on actual observed sizes
    pub fn update(&mut self, mapping_size: usize, sequence_size: usize) {
        // Use exponential moving average to adapt to actual sizes
        self.mapping_pairs = (self.mapping_pairs + mapping_size) / 2;
        self.sequence_items = (self.sequence_items + sequence_size) / 2;
    }
}

impl Default for CapacityHints {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-copy string wrapper that borrows when possible
///
/// This avoids allocating strings when we can reference the source data directly.
/// Falls back to owned strings when necessary (e.g., escape processing).
pub type ZeroCopyStr<'a> = Cow<'a, str>;

/// String pool for deduplicating common strings during parsing
///
/// This is similar to string interning but specifically for parsing performance.
#[cfg(feature = "std")]
use crate::utils::string_interner::StringInterner;

/// Performance optimizer that combines multiple optimization strategies
#[derive(Debug)]
pub struct PerformanceOptimizer {
    /// Capacity hints for allocation
    pub hints: CapacityHints,
    /// String interner for deduplication
    #[cfg(feature = "std")]
    pub string_interner: Option<StringInterner>,
    /// Whether to use lazy tag coercion
    pub lazy_tags: bool,
    /// Whether to use zero-copy strings where possible
    pub zero_copy: bool,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer with default settings
    pub fn new() -> Self {
        Self {
            hints: CapacityHints::new(),
            #[cfg(feature = "std")]
            string_interner: None,
            lazy_tags: false,
            zero_copy: false,
        }
    }

    /// Enable all optimizations
    pub fn aggressive() -> Self {
        Self {
            hints: CapacityHints::large(),
            #[cfg(feature = "std")]
            string_interner: Some(StringInterner::with_capacity(256)),
            lazy_tags: true,
            zero_copy: true,
        }
    }

    /// Enable string pooling
    #[cfg(feature = "std")]
    pub fn enable_string_interning(&mut self, capacity: usize) {
        self.string_interner = Some(StringInterner::with_capacity(capacity));
    }

    /// Enable lazy tag coercion
    pub fn enable_lazy_tags(&mut self) {
        self.lazy_tags = true;
    }

    /// Enable zero-copy strings
    pub fn enable_zero_copy(&mut self) {
        self.zero_copy = true;
    }

    /// Pre-allocate a vector with appropriate capacity
    pub fn alloc_vec<T>(&self) -> Vec<T> {
        Vec::with_capacity(self.hints.sequence_items)
    }

    /// Pre-allocate a string with appropriate capacity
    pub fn alloc_string(&self) -> String {
        String::with_capacity(self.hints.string_capacity)
    }
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast path detector for common YAML patterns
///
/// Detects patterns that can be handled with optimized code paths.
pub struct FastPathDetector;

impl FastPathDetector {
    /// Check if a string is a simple unquoted scalar (no special characters)
    pub fn is_simple_scalar(s: &str) -> bool {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    /// Check if a string is a simple integer
    pub fn is_simple_int(s: &str) -> bool {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_digit() || (c == '-' && s.starts_with('-')))
    }

    /// Check if a line is likely to be a simple key-value pair
    pub fn is_simple_mapping_line(s: &str) -> bool {
        s.contains(crate::constants::CHAR_COLON)
            && !s.contains(crate::constants::CHAR_HASH)
            && !s.contains(crate::constants::CHAR_LBRACKET)
            && !s.contains(crate::constants::CHAR_LBRACE)
    }

    /// Detect if we can use a fast path for this document structure
    pub fn can_use_fast_path(content: &str) -> bool {
        // Simple heuristic: if document has no complex features, use fast path
        !content.contains(crate::constants::STR_DOC_START) // No document markers
            && !content.contains(crate::constants::CHAR_AMPERSAND) // No anchors
            && !content.contains(crate::constants::CHAR_ASTERISK) // No aliases
            && !content.contains("!!") // No explicit tags
            && !content.contains(crate::constants::STR_LITERAL_BLOCK) // No literal scalars
            && !content.contains(crate::constants::STR_FOLDED_BLOCK) // No folded scalars
    }
}

/// Memory-efficient node builder that reuses allocations
#[cfg(feature = "alloc")]
pub struct NodeBuilder {
    /// Capacity hints
    hints: CapacityHints,
    /// Reusable string buffer
    string_buffer: String,
    /// Reusable vec buffer for sequences
    #[allow(dead_code)]
    vec_buffer: Vec<Node>,
}

#[cfg(feature = "alloc")]
impl NodeBuilder {
    /// Create a new node builder
    pub fn new() -> Self {
        let hints = CapacityHints::new();
        Self {
            string_buffer: String::with_capacity(hints.string_capacity),
            vec_buffer: Vec::with_capacity(hints.sequence_items),
            hints,
        }
    }

    /// Create a node builder with custom capacity hints
    pub fn with_hints(hints: CapacityHints) -> Self {
        Self {
            string_buffer: String::with_capacity(hints.string_capacity),
            vec_buffer: Vec::with_capacity(hints.sequence_items),
            hints,
        }
    }

    /// Build a string node, reusing the internal buffer
    pub fn build_string(&mut self, value: &str) -> Node {
        self.string_buffer.clear();
        self.string_buffer.push_str(value);
        Node::Str(
            self.string_buffer.clone(),
            crate::QuoteType::Unquoted,
            crate::BlockStyle::None,
        )
    }

    /// Build an array node with pre-allocated capacity
    pub fn build_array_with_capacity(&self, capacity: usize) -> Node {
        Node::Array(Vec::with_capacity(capacity))
    }

    /// Build a mapping node with pre-allocated capacity
    pub fn build_mapping_with_capacity(&self, capacity: usize) -> Node {
        Node::Mapping(Vec::with_capacity(capacity))
    }

    /// Get capacity hints
    pub fn hints(&self) -> &CapacityHints {
        &self.hints
    }

    /// Update capacity hints based on observed usage
    pub fn update_hints(&mut self, mapping_size: usize, sequence_size: usize) {
        self.hints.update(mapping_size, sequence_size);
    }
}

#[cfg(feature = "alloc")]
impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_tag_int() {
        let mut lazy = LazyTag::new("42".to_string(), "!!int".to_string());
        assert!(!lazy.is_coerced());

        {
            let node = lazy.get_or_coerce();
            assert!(matches!(node, Node::Number(crate::Numeric::Integer(42))));
        }
        assert!(lazy.is_coerced());
    }

    #[test]
    fn test_lazy_tag_bool() {
        let mut lazy = LazyTag::new("true".to_string(), "!!bool".to_string());
        let node = lazy.get_or_coerce();
        assert!(matches!(node, Node::Boolean(true)));
    }

    #[test]
    fn test_capacity_hints() {
        let hints = CapacityHints::new();
        assert_eq!(hints.mapping_pairs, 8);
        assert_eq!(hints.sequence_items, 8);

        let small = CapacityHints::small();
        assert_eq!(small.mapping_pairs, 4);

        let large = CapacityHints::large();
        assert_eq!(large.mapping_pairs, 32);
    }

    #[test]
    fn test_capacity_hints_update() {
        let mut hints = CapacityHints::new();
        hints.update(16, 20);
        assert!(hints.mapping_pairs > 8);
        assert!(hints.sequence_items > 8);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_string_interning() {
        let interner = StringInterner::new();

        let s1 = interner.intern("test");
        let s2 = interner.intern("test");

        assert_eq!(s1.as_str(), s2.as_str());
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_fast_path_detector() {
        assert!(FastPathDetector::is_simple_scalar("hello"));
        assert!(FastPathDetector::is_simple_scalar("hello_world"));
        assert!(!FastPathDetector::is_simple_scalar("hello world"));

        assert!(FastPathDetector::is_simple_int("123"));
        assert!(FastPathDetector::is_simple_int("-456"));
        assert!(!FastPathDetector::is_simple_int("12.34"));

        assert!(FastPathDetector::is_simple_mapping_line("key: value"));
        assert!(!FastPathDetector::is_simple_mapping_line("key: [1, 2, 3]"));
    }

    #[test]
    fn test_performance_optimizer() {
        let optimizer = PerformanceOptimizer::new();
        assert!(!optimizer.lazy_tags);
        assert!(!optimizer.zero_copy);

        let aggressive = PerformanceOptimizer::aggressive();
        assert!(aggressive.lazy_tags);
        assert!(aggressive.zero_copy);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_builder() {
        let mut builder = NodeBuilder::new();
        let _node = builder.build_string("test");
        let _array = builder.build_array_with_capacity(10);
        let _mapping = builder.build_mapping_with_capacity(5);
    }

    #[test]
    fn test_fast_path_detection() {
        let simple = "name: John\nage: 30";
        assert!(FastPathDetector::can_use_fast_path(simple));

        let complex = "name: John\n---\nage: 30";
        assert!(!FastPathDetector::can_use_fast_path(complex));
    }
}
