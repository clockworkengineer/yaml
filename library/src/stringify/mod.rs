//! YAML Stringification Modules
//!
//! Aggregates modules for converting YAML nodes to various string representations, including
//! Bencode, default YAML, JSON, TOML, XML, and streaming/custom serializers. Provides formatting options.
//!
//! Copyright (c) 2026 YAML Library Developers

/// bencode
pub mod bencode;
/// default
pub mod default;
/// Formatting options and control
#[cfg(feature = "alloc")]
pub mod format;
/// json
pub mod json;
/// Custom serializer support
#[cfg(feature = "alloc")]
pub mod serializer;
/// Streaming serialization
#[cfg(feature = "alloc")]
pub mod streaming;
/// toml
pub mod toml;
/// xml
pub mod xml;
