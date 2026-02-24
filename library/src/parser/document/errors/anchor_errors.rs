
/*
 * Anchor & Alias Error Helpers
 *
 * Centralizes error construction for YAML anchors and aliases, providing helpers
 * for consistent error messages and easier future maintenance.
 *
 * Copyright (c) 2026 YAML Library Developers
 */

use crate::parser::token_stream::TokenStream;

/// Centralized, behavior-identical error construction for anchors and aliases.
///
/// This module provides helpers that return the exact strings currently used
/// across call sites, making future tweaks easier without changing semantics.
pub struct AnchorErrors;

impl AnchorErrors {
    /// SR86 / SU74: Anchors cannot be applied to alias nodes
    pub fn invalid_anchored_alias(stream: &mut TokenStream) -> crate::error::YamlError {
        crate::parser::document::error_builder::mapping_key_error_yaml(
            stream.source_mut(),
            "Invalid anchored alias: anchors cannot be applied to alias nodes",
        )
    }

    /// SY6V: Anchor cannot precede a block sequence indicator on the same line
    pub fn anchor_cannot_precede_dash_same_line(
        stream: &mut TokenStream,
    ) -> crate::error::YamlError {
        crate::parser::document::error_builder::syntax_error(
            stream.source_mut(),
            "Invalid anchor usage: anchor cannot directly precede a sequence indicator on the same line; attach the anchor to the node (e.g., '- &name value')",
        )
    }

    /// SY6V: Anchor cannot precede a block sequence indicator in block context (value parser)
    pub fn anchor_cannot_precede_dash_block(stream: &mut TokenStream) -> crate::error::YamlError {
        crate::parser::document::error_builder::syntax_error(
            stream.source_mut(),
            "Invalid anchor usage: anchor cannot directly precede a sequence indicator; attach the anchor to the node (e.g., '- &name value')",
        )
    }

    /// Multiple anchors applied to the same node (defensive message)
    pub fn multiple_anchors(stream: &mut TokenStream) -> crate::error::YamlError {
        crate::parser::document::error_builder::mapping_key_error_yaml(
            stream.source_mut(),
            "A node cannot have multiple anchors",
        )
    }
}
