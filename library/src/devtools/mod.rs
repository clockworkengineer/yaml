//! Developer tools for debugging and inspection
//!
//! This module provides utilities for developers working with YAML documents,
//! including pretty-printing, inspection, debugging, and diagnostics.

#[cfg(feature = "alloc")]
pub mod debug;
#[cfg(feature = "alloc")]
pub mod diff;
#[cfg(feature = "alloc")]
pub mod inspect;
#[cfg(feature = "alloc")]
pub mod trace;
