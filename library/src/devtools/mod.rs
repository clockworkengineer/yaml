//! YAML Developer Tools Module
//!
//! Aggregates developer utilities for debugging, inspection, pretty-printing, and diagnostics
//! for YAML documents. Includes submodules for debugging, diffing, inspection, and tracing.
//!
//! Copyright (c) 2026 YAML Library Developers

#[cfg(feature = "alloc")]
pub mod debug;
#[cfg(feature = "alloc")]
pub mod diff;
#[cfg(feature = "alloc")]
pub mod inspect;
#[cfg(feature = "alloc")]
pub mod trace;
