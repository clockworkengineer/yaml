//! Validation and schema support for YAML documents
//!
//! This module provides validation capabilities including:
//! - Schema definition for YAML structures
//! - Built-in validators for common types and constraints
//! - Custom validation rules
//! - Comprehensive error reporting

pub mod engine;
pub mod error;
pub mod schema;
pub mod validators;

pub use engine::{SchemaValidator, ValidationContext, ValidationError};
pub use schema::{ArraySchema, ObjectSchema, PropertySchema, Schema, SchemaType};
pub use validators::{
    CustomValidator, EnumValidator, LengthValidator, PatternValidator, RangeValidator,
    RequiredValidator, TypeValidator, ValidationResult, Validator,
};
