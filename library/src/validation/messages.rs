//! Shared validation message helpers (behavior-neutral DRY refactor)
//!
//! This module centralizes common human-readable validation descriptions
//! so that range/length/enum messages stay consistent across validators
//! while keeping the exact existing wording.

use alloc::string::String;
use alloc::vec::Vec;

use crate::validation::schema::SchemaType;

/// Description for type expectations.
pub fn type_must_be(expected: &SchemaType) -> String {
    format!("Type must be {:?}", expected)
}

/// Descriptions for numeric range validators.
pub fn value_must_be_between(min: f64, max: f64) -> String {
    format!("Value must be between {} and {}", min, max)
}

pub fn value_must_be_at_least(min: f64) -> String {
    format!("Value must be at least {}", min)
}

pub fn value_must_be_at_most(max: f64) -> String {
    format!("Value must be at most {}", max)
}

pub fn no_range_restriction() -> String {
    "No range restriction".to_string()
}

/// Descriptions for length validators.
pub fn length_must_be_between(min: usize, max: usize) -> String {
    format!("Length must be between {} and {}", min, max)
}

pub fn length_must_be_at_least(min: usize) -> String {
    format!("Length must be at least {}", min)
}

pub fn length_must_be_at_most(max: usize) -> String {
    format!("Length must be at most {}", max)
}

pub fn no_length_restriction() -> String {
    "No length restriction".to_string()
}

/// Description for enum validators.
pub fn must_be_one_of(allowed: &Vec<String>) -> String {
    format!("Must be one of: {}", allowed.join(", "))
}
