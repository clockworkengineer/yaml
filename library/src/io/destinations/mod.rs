//! Output Destinations Module
//!
//! Aggregates buffer and file-based destinations for writing encoded YAML or JSON data.
//! Provides unified interfaces for memory and disk output implementations.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Module providing a buffer-based destination for writing JSON data into memory
pub mod buffer;
/// Module providing a file-based destination for writing JSON data to disk
pub mod file;
