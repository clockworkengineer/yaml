
/*
 * Indentation Error Helpers
 *
 * Centralizes error construction for YAML indentation parsing, providing helpers
 * for consistent error messages and easier future maintenance.
 *
 * Copyright (c) 2026 YAML Library Developers
 */

use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::parser::document::error_builder;

/// Centralized constructors for indentation-related parsing errors.
/// Behavior-neutral: messages and error categories match existing call sites.
pub struct IndentationErrors;

impl IndentationErrors {
    /// Tabs are forbidden as indentation in general YAML block context.
    /// Matches existing forbidden_error usage (Syntax category).
    pub fn tabs_not_allowed_yaml_syntax(source: &mut dyn ISource) -> YamlError {
        error_builder::forbidden_error(source, "Tabs", "as indentation in YAML")
    }

    /// Tabs are forbidden as indentation inside flow collections.
    /// Matches existing forbidden_error usage (Syntax category).
    pub fn tabs_not_allowed_flow_collections(source: &mut dyn ISource) -> YamlError {
        error_builder::forbidden_error(source, "Tabs", "as indentation in YAML flow collections")
    }

    /// Tabs used for indentation in block context surfaced via dedicated indentation error.
    /// Matches existing tab_indentation_error_yaml (Indentation category).
    pub fn tabs_not_allowed_yaml_block(source: &mut dyn ISource) -> YamlError {
        error_builder::tab_indentation_error_yaml(source)
    }
}
