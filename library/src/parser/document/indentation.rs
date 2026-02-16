//! Module: parser/document/indentation.rs
//!
//! Centralizes indentation validation policies so future changes to
//! indentation rules can be made in one place without touching many
//! parser call sites.
//!
//! This module provides small helpers that produce structured `YamlError`s
//! via the shared error builder, including source context (current char,
//! indent level). It also offers context-aware variants that can leverage
//! `ParsingContext` when needed.

use crate::error::{ErrorKind, YamlError};
use crate::io::traits::ISource;
use crate::parser::ParseResult;
use crate::parser::document::context::ParsingContext;

/// Ensures that `actual` indent is at least `expected`.
///
/// Returns a structured indentation error with source context when the
/// requirement is not met. This mirrors existing behavior but centralizes
/// message construction and policy.
pub fn ensure_indent_at_least(
    source: &mut dyn ISource,
    actual: usize,
    expected: usize,
    what: &str,
) -> ParseResult<()> {
    if actual < expected {
        return Err(crate::parser::document::error_builder::indentation_error(
            source,
            &format!(
                "{} at invalid indentation: expected >= {}, got {}",
                what, expected, actual
            ),
        ));
    }
    Ok(())
}

/// Lightweight variant that avoids borrowing the source; useful where
/// a `TokenStream` already holds a mutable borrow of the source.
pub fn ensure_indent_at_least_no_source(
    actual: usize,
    expected: usize,
    what: &str,
) -> ParseResult<()> {
    if actual < expected {
        return Err(YamlError::new(
            ErrorKind::ValidationError,
            format!(
                "{} at invalid indentation: expected >= {}, got {}",
                what, expected, actual
            ),
        ));
    }
    Ok(())
}

/// Context-aware child indentation validation.
///
/// Uses `ParsingContext` parent indent to validate `child_indent` is deeper
/// than its parent. This is not wired into existing code paths yet, but it
/// provides a single API for future indentation policy tweaks.
#[allow(dead_code)]
pub fn ensure_valid_child_indent(
    source: &mut dyn ISource,
    ctx: &ParsingContext,
    child_indent: usize,
    what: &str,
) -> Result<(), YamlError> {
    if !ctx.is_valid_child_indent(child_indent) {
        return Err(crate::parser::document::error_builder::indentation_error(
            source,
            &format!(
                "Invalid child indentation for {}: parent {}, child {}",
                what, ctx.indent_level, child_indent
            ),
        ));
    }
    Ok(())
}
