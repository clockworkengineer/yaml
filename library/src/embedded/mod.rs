//! Embedded Systems Support Module
//!
//! Aggregates compile-time configuration, memory limits, and lightweight alternatives for YAML parsing
//! on resource-constrained embedded environments. Includes numeric type recommendations and memory optimization tips.
//!
//! Copyright (c) 2026 YAML Library Developers

//! - `Numeric::Float(f64)`
//! - `Numeric::UInteger(u64)`
//!
//! ## Helper Methods
//!
//! When the `embedded` feature is enabled, `Numeric` provides helper methods:
//! - `to_i32()` - Convert any numeric to i32 (returns Option)
//! - `to_f32()` - Convert any numeric to f32
//! - `fits_in_i32()` - Check if value can be represented as i32
//! - `size_bytes()` - Get memory footprint of the variant
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use yaml_lib::nodes::node::{Node, Numeric};
//!
//! // Prefer Int32 instead of Integer for embedded
//! let small_num = Node::Number(Numeric::Int32(42));  // 4 bytes
//! // Instead of: Node::Number(Numeric::Integer(42))  // 8 bytes
//!
//! // Convert existing numbers to embedded-friendly types
//! let num = Numeric::Integer(123);
//! if let Some(i32_val) = num.to_i32() {
//!     // Use i32_val in embedded context
//! }
//!
//! // Check memory footprint
//! let size = num.size_bytes();  // Returns 8 for Integer variant
//! ```
//!
//! ## Memory Savings
//!
//! By using Int32 instead of Integer throughout your code:
//! - 50% reduction in numeric storage (4 bytes vs 8 bytes)
//! - Better cache utilization on 32-bit embedded processors
//! - Reduced heap pressure in constrained environments

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
