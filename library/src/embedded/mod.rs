//! Module: embedded/mod.rs
//!
//! Embedded systems support module providing compile-time configuration,
//! memory limits, and lightweight alternatives for resource-constrained environments.

/// Configuration constants for embedded systems
pub mod config;

/// Runtime and compile-time limits for embedded systems
pub mod limits;

/// Lightweight node representation for embedded systems
#[cfg(feature = "embedded")]
pub mod lightweight_node;

/// Custom allocator support for embedded systems
#[cfg(feature = "embedded")]
pub mod allocator;
