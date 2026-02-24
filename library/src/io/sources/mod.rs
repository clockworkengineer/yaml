
//! Input Sources Module
//!
//! Aggregates buffer and file-based sources for reading YAML or JSON data.
//! Provides unified interfaces for memory and disk input implementations.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Module providing a buffer-based source for reading JSON data from memory
pub mod buffer;
/// Module providing a file-based source for reading JSON data from disk
pub mod file;
