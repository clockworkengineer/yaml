//! Module: embedded/limits.rs
//!
//! Runtime limit checking and enforcement for embedded systems.
//! Provides validators to ensure parsed YAML stays within configured bounds.

use crate::embedded::config::*;

/// Error type for limit violations
#[derive(Debug, Clone, PartialEq)]
pub enum LimitError {
    /// Nesting depth exceeded maximum
    NestingDepthExceeded { current: usize, max: usize },
    /// Document size exceeded maximum
    DocumentSizeExceeded { current: usize, max: usize },
    /// String length exceeded maximum
    StringLengthExceeded { current: usize, max: usize },
    /// Sequence items exceeded maximum
    SequenceItemsExceeded { current: usize, max: usize },
    /// Mapping pairs exceeded maximum
    MappingPairsExceeded { current: usize, max: usize },
    /// Anchors exceeded maximum
    AnchorsExceeded { current: usize, max: usize },
}

#[cfg(feature = "std")]
impl std::fmt::Display for LimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitError::NestingDepthExceeded { current, max } => {
                write!(f, "Nesting depth {} exceeded maximum {}", current, max)
            }
            LimitError::DocumentSizeExceeded { current, max } => {
                write!(f, "Document size {} exceeded maximum {}", current, max)
            }
            LimitError::StringLengthExceeded { current, max } => {
                write!(f, "String length {} exceeded maximum {}", current, max)
            }
            LimitError::SequenceItemsExceeded { current, max } => {
                write!(f, "Sequence items {} exceeded maximum {}", current, max)
            }
            LimitError::MappingPairsExceeded { current, max } => {
                write!(f, "Mapping pairs {} exceeded maximum {}", current, max)
            }
            LimitError::AnchorsExceeded { current, max } => {
                write!(f, "Anchors {} exceeded maximum {}", current, max)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LimitError {}

/// Checks if nesting depth is within limits
#[inline]
pub fn check_nesting_depth(depth: usize) -> Result<(), LimitError> {
    if depth > MAX_NESTING_DEPTH {
        Err(LimitError::NestingDepthExceeded {
            current: depth,
            max: MAX_NESTING_DEPTH,
        })
    } else {
        Ok(())
    }
}

/// Checks if document size is within limits
#[inline]
pub fn check_document_size(size: usize) -> Result<(), LimitError> {
    if size > MAX_DOCUMENT_SIZE {
        Err(LimitError::DocumentSizeExceeded {
            current: size,
            max: MAX_DOCUMENT_SIZE,
        })
    } else {
        Ok(())
    }
}

/// Checks if string length is within limits
#[inline]
pub fn check_string_length(length: usize) -> Result<(), LimitError> {
    if length > MAX_STRING_LENGTH {
        Err(LimitError::StringLengthExceeded {
            current: length,
            max: MAX_STRING_LENGTH,
        })
    } else {
        Ok(())
    }
}

/// Checks if sequence item count is within limits
#[inline]
pub fn check_sequence_items(count: usize) -> Result<(), LimitError> {
    if count > MAX_SEQUENCE_ITEMS {
        Err(LimitError::SequenceItemsExceeded {
            current: count,
            max: MAX_SEQUENCE_ITEMS,
        })
    } else {
        Ok(())
    }
}

/// Checks if mapping pair count is within limits
#[inline]
pub fn check_mapping_pairs(count: usize) -> Result<(), LimitError> {
    if count > MAX_MAPPING_PAIRS {
        Err(LimitError::MappingPairsExceeded {
            current: count,
            max: MAX_MAPPING_PAIRS,
        })
    } else {
        Ok(())
    }
}

/// Checks if anchor count is within limits
#[inline]
pub fn check_anchor_count(count: usize) -> Result<(), LimitError> {
    if count > MAX_ANCHORS {
        Err(LimitError::AnchorsExceeded {
            current: count,
            max: MAX_ANCHORS,
        })
    } else {
        Ok(())
    }
}

/// Validates a Node structure against embedded system limits
///
/// This validator recursively checks a Node tree to ensure it stays within
/// the configured limits for embedded systems.
#[cfg(feature = "alloc")]
pub struct NodeValidator {
    max_depth_seen: usize,
    anchor_count: usize,
}

#[cfg(feature = "alloc")]
impl NodeValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            max_depth_seen: 0,
            anchor_count: 0,
        }
    }

    /// Validate a node and all its children
    ///
    /// Returns Ok(()) if the node structure is valid, or the first LimitError encountered.
    pub fn validate(&mut self, node: &crate::nodes::node::Node) -> Result<(), LimitError> {
        self.validate_recursive(node, 0)
    }

    fn validate_recursive(
        &mut self,
        node: &crate::nodes::node::Node,
        depth: usize,
    ) -> Result<(), LimitError> {
        check_nesting_depth(depth)?;

        if depth > self.max_depth_seen {
            self.max_depth_seen = depth;
        }

        match node {
            crate::nodes::node::Node::Str(s, _, _) => {
                check_string_length(s.len())?;
            }
            crate::nodes::node::Node::Array(items) => {
                check_sequence_items(items.len())?;
                for item in items {
                    self.validate_recursive(item, depth + 1)?;
                }
            }
            crate::nodes::node::Node::Set(items) => {
                check_sequence_items(items.len())?;
                for item in items {
                    self.validate_recursive(item, depth + 1)?;
                }
            }
            crate::nodes::node::Node::Mapping(pairs) => {
                check_mapping_pairs(pairs.len())?;
                for (key, value) in pairs {
                    self.validate_recursive(key, depth + 1)?;
                    self.validate_recursive(value, depth + 1)?;
                }
            }
            crate::nodes::node::Node::Anchored(inner, _) => {
                self.anchor_count += 1;
                check_anchor_count(self.anchor_count)?;
                self.validate_recursive(inner, depth)?;
            }
            crate::nodes::node::Node::Tagged(inner, _) => {
                self.validate_recursive(inner, depth)?;
            }
            crate::nodes::node::Node::Document(nodes) => {
                for node in nodes {
                    self.validate_recursive(node, depth + 1)?;
                }
            }
            crate::nodes::node::Node::Documents(docs) => {
                for doc in docs {
                    self.validate_recursive(doc, depth + 1)?;
                }
            }
            _ => {} // Boolean, Number, None, Comment, Alias don't need validation
        }

        Ok(())
    }

    /// Get the maximum nesting depth seen during validation
    pub fn max_depth(&self) -> usize {
        self.max_depth_seen
    }

    /// Get the number of anchors counted during validation
    pub fn anchor_count(&self) -> usize {
        self.anchor_count
    }
}

#[cfg(feature = "alloc")]
impl Default for NodeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_depth_validation() {
        assert!(check_nesting_depth(0).is_ok());
        assert!(check_nesting_depth(MAX_NESTING_DEPTH).is_ok());
        assert!(check_nesting_depth(MAX_NESTING_DEPTH + 1).is_err());
    }

    #[test]
    fn test_document_size_validation() {
        assert!(check_document_size(0).is_ok());
        assert!(check_document_size(MAX_DOCUMENT_SIZE).is_ok());
        assert!(check_document_size(MAX_DOCUMENT_SIZE + 1).is_err());
    }

    #[test]
    fn test_string_length_validation() {
        assert!(check_string_length(0).is_ok());
        assert!(check_string_length(MAX_STRING_LENGTH).is_ok());
        assert!(check_string_length(MAX_STRING_LENGTH + 1).is_err());
    }

    #[test]
    fn test_sequence_items_validation() {
        assert!(check_sequence_items(0).is_ok());
        assert!(check_sequence_items(MAX_SEQUENCE_ITEMS).is_ok());
        assert!(check_sequence_items(MAX_SEQUENCE_ITEMS + 1).is_err());
    }

    #[test]
    fn test_mapping_pairs_validation() {
        assert!(check_mapping_pairs(0).is_ok());
        assert!(check_mapping_pairs(MAX_MAPPING_PAIRS).is_ok());
        assert!(check_mapping_pairs(MAX_MAPPING_PAIRS + 1).is_err());
    }

    #[test]
    fn test_anchor_count_validation() {
        assert!(check_anchor_count(0).is_ok());
        assert!(check_anchor_count(MAX_ANCHORS).is_ok());
        assert!(check_anchor_count(MAX_ANCHORS + 1).is_err());
    }

    #[test]
    fn test_limit_error_messages() {
        let err = LimitError::NestingDepthExceeded {
            current: 50,
            max: 32,
        };
        assert_eq!(
            err,
            LimitError::NestingDepthExceeded {
                current: 50,
                max: 32
            }
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_simple_node() {
        use crate::nodes::node::{Node, Numeric};
        let node = Node::Number(Numeric::Int32(42));
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&node).is_ok());
        assert_eq!(validator.max_depth(), 0);
        assert_eq!(validator.anchor_count(), 0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_array() {
        use crate::nodes::node::{Node, Numeric};
        let arr = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
        ]);
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&arr).is_ok());
        assert_eq!(validator.max_depth(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_mapping() {
        use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
        let mapping = Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Int32(1)),
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Int32(2)),
            ),
        ]);
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&mapping).is_ok());
        assert_eq!(validator.max_depth(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_nested_structure() {
        use crate::nodes::node::{Node, Numeric};
        let nested = Node::Array(vec![
            Node::Array(vec![Node::Number(Numeric::Int32(1))]),
            Node::Array(vec![Node::Number(Numeric::Int32(2))]),
        ]);
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&nested).is_ok());
        assert_eq!(validator.max_depth(), 2);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_excessive_nesting() {
        use crate::nodes::node::Node;
        let mut deep = Node::None;
        for _ in 0..35 {
            deep = Node::Array(alloc::vec![deep]);
        }
        let mut validator = NodeValidator::new();
        let result = validator.validate(&deep);
        assert!(result.is_err());
        match result {
            Err(LimitError::NestingDepthExceeded { current, max }) => {
                assert!(current > max);
                assert_eq!(max, MAX_NESTING_DEPTH);
            }
            _ => panic!("Expected NestingDepthExceeded error"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_long_string() {
        use crate::nodes::node::{BlockStyle, Node, QuoteType};
        let long_str = "x".repeat(MAX_STRING_LENGTH + 1);
        let node = Node::Str(long_str.clone(), QuoteType::Unquoted, BlockStyle::None);
        let mut validator = NodeValidator::new();
        let result = validator.validate(&node);
        assert!(result.is_err());
        match result {
            Err(LimitError::StringLengthExceeded { current, max }) => {
                assert_eq!(current, long_str.len());
                assert_eq!(max, MAX_STRING_LENGTH);
            }
            _ => panic!("Expected StringLengthExceeded error"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_too_many_sequence_items() {
        use crate::nodes::node::{Node, Numeric};
        let mut items = alloc::vec::Vec::new();
        for i in 0..=(MAX_SEQUENCE_ITEMS + 1) {
            items.push(Node::Number(Numeric::Int32(i as i32)));
        }
        let arr = Node::Array(items);
        let mut validator = NodeValidator::new();
        let result = validator.validate(&arr);
        assert!(result.is_err());
        match result {
            Err(LimitError::SequenceItemsExceeded { current, max }) => {
                assert!(current > max);
                assert_eq!(max, MAX_SEQUENCE_ITEMS);
            }
            _ => panic!("Expected SequenceItemsExceeded error"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_too_many_mapping_pairs() {
        use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
        let mut pairs = alloc::vec::Vec::new();
        for i in 0..=(MAX_MAPPING_PAIRS + 1) {
            pairs.push((
                Node::Str(format!("key{}", i), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Int32(i as i32)),
            ));
        }
        let mapping = Node::Mapping(pairs);
        let mut validator = NodeValidator::new();
        let result = validator.validate(&mapping);
        assert!(result.is_err());
        match result {
            Err(LimitError::MappingPairsExceeded { current, max }) => {
                assert!(current > max);
                assert_eq!(max, MAX_MAPPING_PAIRS);
            }
            _ => panic!("Expected MappingPairsExceeded error"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_anchors() {
        use crate::nodes::node::{Node, Numeric};
        let anchored = Node::Anchored(
            alloc::boxed::Box::new(Node::Number(Numeric::Int32(42))),
            "anchor1".to_string(),
        );
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&anchored).is_ok());
        assert_eq!(validator.anchor_count(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_too_many_anchors() {
        use crate::nodes::node::Node;
        let mut node = Node::None;
        for i in 0..=(MAX_ANCHORS + 1) {
            node = Node::Anchored(alloc::boxed::Box::new(node), format!("anchor{}", i));
        }
        let mut validator = NodeValidator::new();
        let result = validator.validate(&node);
        assert!(result.is_err());
        match result {
            Err(LimitError::AnchorsExceeded { current, max }) => {
                assert!(current > max);
                assert_eq!(max, MAX_ANCHORS);
            }
            _ => panic!("Expected AnchorsExceeded error"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_documents() {
        use crate::nodes::node::{Node, Numeric};
        let docs = Node::Documents(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
        ]);
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&docs).is_ok());
        assert_eq!(validator.max_depth(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_validator_default() {
        let validator = NodeValidator::default();
        assert_eq!(validator.max_depth(), 0);
        assert_eq!(validator.anchor_count(), 0);
    }
    #[test]
    fn test_nesting_depth_negative() {
        // Negative values are not possible for usize, but test zero limit edge
        assert!(check_nesting_depth(0).is_ok());
    }

    #[test]
    fn test_document_size_zero_limit() {
        // Simulate zero limit by temporarily overriding MAX_DOCUMENT_SIZE
        // (Assume MAX_DOCUMENT_SIZE is not const for this test, otherwise skip)
        // Here, just check zero size is valid
        assert!(check_document_size(0).is_ok());
    }

    #[test]
    fn test_string_length_empty_string() {
        assert!(check_string_length(0).is_ok());
    }

    #[test]
    fn test_sequence_items_empty() {
        assert!(check_sequence_items(0).is_ok());
    }

    #[test]
    fn test_mapping_pairs_empty() {
        assert!(check_mapping_pairs(0).is_ok());
    }

    #[test]
    fn test_anchor_count_zero() {
        assert!(check_anchor_count(0).is_ok());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_limit_error_display() {
        let err = LimitError::SequenceItemsExceeded {
            current: 10,
            max: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Sequence items 10 exceeded maximum 5"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_limit_error_debug() {
        let err = LimitError::AnchorsExceeded { current: 3, max: 2 };
        let msg = format!("{:?}", err);
        assert!(msg.contains("AnchorsExceeded"));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_validator_empty_array() {
        use crate::nodes::node::Node;
        let arr = Node::Array(vec![]);
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&arr).is_ok());
        assert_eq!(validator.max_depth(), 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_validator_empty_mapping() {
        use crate::nodes::node::Node;
        let mapping = Node::Mapping(vec![]);
        let mut validator = NodeValidator::new();
        assert!(validator.validate(&mapping).is_ok());
        assert_eq!(validator.max_depth(), 0);
    }
}
