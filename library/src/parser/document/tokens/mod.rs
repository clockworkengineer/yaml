//! YAML Token Parsing Module
//!
//! Aggregates token-based parsing logic for the YAML document parser, including
//! inline, mapping, sequence, and value token handling submodules.
//!
//! Copyright (c) 2026 YAML Library Developers

pub mod inline;
pub mod mapping;
pub mod sequence;
pub mod value;
