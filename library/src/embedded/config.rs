//! Embedded Configuration Constants
//!
//! Compile-time configuration for YAML parsing on embedded systems.
//! Provides tunable constants for nesting depth, document size, and string length.
//! Override defaults at compile time or via builder patterns for resource-constrained environments.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Maximum depth of nested YAML structures (sequences, mappings)
/// Default: 32 levels
/// For embedded systems with tight constraints, consider reducing to 8-16
pub const MAX_NESTING_DEPTH: usize = 32;

/// Maximum size of a single YAML document in bytes
/// Default: 64KB
/// For embedded systems, consider reducing to 4KB-16KB
pub const MAX_DOCUMENT_SIZE: usize = 65536;

/// Maximum length of a single string value
/// Default: 4KB
/// For embedded systems, consider reducing to 256-1024 bytes
pub const MAX_STRING_LENGTH: usize = 4096;

/// Maximum number of items in a sequence
/// Default: 256 items
/// For embedded systems, consider reducing to 32-64 items
pub const MAX_SEQUENCE_ITEMS: usize = 256;

/// Maximum number of key-value pairs in a mapping
/// Default: 256 pairs
/// For embedded systems, consider reducing to 32-64 pairs
pub const MAX_MAPPING_PAIRS: usize = 256;

/// Maximum number of anchor definitions
/// Default: 128 anchors
/// For embedded systems, consider reducing to 16-32 anchors
pub const MAX_ANCHORS: usize = 128;

/// Enable/disable validation checks
/// Set to false in embedded systems to save code size and improve performance
pub const ENABLE_VALIDATION: bool = true;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_constants_have_reasonable_defaults() {
        assert!(MAX_NESTING_DEPTH > 0);
        assert!(MAX_DOCUMENT_SIZE > 0);
        assert!(MAX_STRING_LENGTH > 0);
        assert!(MAX_SEQUENCE_ITEMS > 0);
        assert!(MAX_MAPPING_PAIRS > 0);
        assert!(MAX_ANCHORS > 0);
    }

    #[test]
    fn test_nesting_depth_is_reasonable() {
        assert!(
            MAX_NESTING_DEPTH >= 8,
            "Nesting depth should support at least 8 levels"
        );
        assert!(
            MAX_NESTING_DEPTH <= 1024,
            "Nesting depth should not exceed 1024 levels"
        );
    }

    #[test]
    fn test_document_size_is_reasonable() {
        assert!(
            MAX_DOCUMENT_SIZE >= 1024,
            "Document size should be at least 1KB"
        );
    }

    #[test]
    fn test_string_length_is_reasonable() {
        assert!(
            MAX_STRING_LENGTH >= 256,
            "String length should be at least 256 bytes"
        );
        assert!(
            MAX_STRING_LENGTH <= 65536,
            "String length should not exceed 64KB"
        );
    }

    #[test]
    fn test_sequence_items_is_reasonable() {
        assert!(
            MAX_SEQUENCE_ITEMS >= 16,
            "Sequence items should be at least 16"
        );
        assert!(
            MAX_SEQUENCE_ITEMS <= 1024,
            "Sequence items should not exceed 1024"
        );
    }

    #[test]
    fn test_mapping_pairs_is_reasonable() {
        assert!(
            MAX_MAPPING_PAIRS >= 16,
            "Mapping pairs should be at least 16"
        );
        assert!(
            MAX_MAPPING_PAIRS <= 1024,
            "Mapping pairs should not exceed 1024"
        );
    }

    #[test]
    fn test_anchors_is_reasonable() {
        assert!(MAX_ANCHORS >= 8, "Anchors should be at least 8");
        assert!(MAX_ANCHORS <= 256, "Anchors should not exceed 256");
    }

    #[test]
    fn test_validation_enabled_by_default() {
        assert!(ENABLE_VALIDATION, "Validation should be enabled by default");
    }

    #[test]
    fn test_nesting_depth_vs_document_size() {
        // If nesting depth is high, document size should be reasonable
        if MAX_NESTING_DEPTH > 128 {
            assert!(
                MAX_DOCUMENT_SIZE >= 8192,
                "High nesting depth should allow larger documents"
            );
        }
    }

    #[test]
    fn test_sequence_items_vs_mapping_pairs() {
        // Sequence and mapping limits should be similar
        let diff = if MAX_SEQUENCE_ITEMS > MAX_MAPPING_PAIRS {
            MAX_SEQUENCE_ITEMS - MAX_MAPPING_PAIRS
        } else {
            MAX_MAPPING_PAIRS - MAX_SEQUENCE_ITEMS
        };
        assert!(
            diff <= 512,
            "Sequence and mapping item limits should not differ by more than 512"
        );
    }

    #[test]
    fn test_string_length_vs_document_size() {
        // String length should not exceed document size
        assert!(
            MAX_STRING_LENGTH <= MAX_DOCUMENT_SIZE,
            "String length should not exceed document size"
        );
    }

    #[test]
    fn test_disable_validation_for_embedded() {
        // Simulate disabling validation for embedded
        let embedded = false;
        let validation = if embedded { false } else { ENABLE_VALIDATION };
        assert_eq!(validation, ENABLE_VALIDATION);
    }
}
