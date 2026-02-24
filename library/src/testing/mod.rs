//! YAML Library Testing Module
//!
//! This module provides utilities and infrastructure for testing the YAML library.
//! It includes support for fuzzing, property-based testing, and safety auditing.
//!
//! # Features
//! - Fuzzing utilities
//! - Property-based testing
//! - Safety auditing tools
//!
//! # Usage
//! Enable the relevant features to access the corresponding testing modules.

#[cfg(feature = "alloc")]
pub mod fuzzing;
#[cfg(feature = "alloc")]
pub mod property;
#[cfg(feature = "alloc")]
pub mod safety;
