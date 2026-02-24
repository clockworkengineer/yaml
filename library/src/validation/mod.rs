//! Validation and Schema Support for YAML Library
//!
//! This module provides validation and schema support for YAML documents. It includes schema
//! definitions, built-in and custom validators, and comprehensive error reporting for robust
//! data validation.
//!
//! # Features
//! - Schema definition for YAML structures
//! - Built-in and custom validators
//! - Comprehensive error reporting
//! - Extensible for custom validation logic
//!
//! # Usage
//! Use these modules to define, validate, and report errors for YAML data structures.

pub mod engine;
pub mod error;
pub mod messages;
pub mod schema;
pub mod validators;

pub use engine::{SchemaValidator, ValidationContext};
pub use error::ValidationError;
pub use schema::{ArraySchema, ObjectSchema, PropertySchema, Schema, SchemaType};
pub use validators::{
    CustomValidator, EnumValidator, LengthValidator, PatternValidator, RangeValidator,
    RequiredValidator, TypeValidator, ValidationResult, Validator,
};
