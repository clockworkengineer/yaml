//! Testing utilities and infrastructure
//!
//! Provides fuzzing, property-based testing, and safety auditing tools.

#[cfg(feature = "alloc")]
pub mod fuzzing;
#[cfg(feature = "alloc")]
pub mod property;
#[cfg(feature = "alloc")]
pub mod safety;
