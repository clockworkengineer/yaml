//! Document Parsing Helpers
//!
//! Provides parsing, validation, and utility helpers for YAML document parsing.
//! Includes entry points for error construction, token stream setup, whitespace/comment skipping,
//! mapping key lookahead, block head classification, indentation validation, and comment parsing.
//!
//! This module is organized into focused sub-modules:
//! - [`core`]             — Error construction, directive handling, token matching
//! - [`document_markers`] — Document start/end marker parsing and classification
//! - [`validation`]       — Indentation, tab, trailing-content, and comment-spacing validation
//! - [`peek_ahead`]       — Mapping key lookahead and block-head classification
//! - [`comments`]         — Comment token parsing
//!
//! Copyright (c) 2026 YAML Library Developers

// DRY NOTE: All comment parsing and comment spacing validation must use parse_comment_token and validate_comment_spacing_token below.
// DRY NOTE: All indentation and tab validation must use validate_indentation_and_whitespace below.
// DRY NOTE: All block head classification (mapping, sequence, value, etc.) must use classify_block_head below.
// DRY NOTE: All mapping key lookahead (colon detection, flow depth) must use peek_ahead_for_mapping_key below.

pub(crate) mod comments;
pub(crate) mod core;
pub(crate) mod document_markers;
pub(crate) mod peek_ahead;
pub(crate) mod validation;

// Re-export everything so callers that use `helpers::foo` continue to work
// without any changes.
#[allow(unused_imports)]
pub(crate) use comments::parse_comment_token;
#[allow(unused_imports)]
pub(crate) use core::{handle_directives, is_token, node_to_inline_string, parse_error_token, to_yaml_error};
pub(crate) use document_markers::{
    classify_doc_marker, parse_document_end_marker, parse_document_markers,
    peek_tag_after_doc_start, DocMarkerKind,
};
#[allow(unused_imports)]
pub(crate) use peek_ahead::{classify_block_head, peek_ahead_for_mapping_key, BlockHeadKind};
#[allow(unused_imports)]
pub(crate) use validation::{
    validate_comment_spacing_token, validate_indentation_and_whitespace,
    validate_no_tab_indentation_tokens, validate_trailing_content_after_document_end,
};
