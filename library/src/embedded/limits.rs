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
}
