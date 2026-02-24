//! YAML I/O Module
//!
//! Aggregates input and output source/destination modules and traits for YAML data processing.
//! Provides unified interfaces for reading from and writing to memory buffers, files, and other I/O targets.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Module containing destination implementations for writing YAML data to different outputs
pub mod destinations;
/// Module containing source implementations for reading YAML data from different inputs
pub mod sources;
/// Module containing trait definitions for YAML I/O operations
pub mod traits;
/// Module containing utility functions for YAML I/O operations
pub mod util;
