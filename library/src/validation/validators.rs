//! Built-in validators for common validation patterns
//!
//! Provides reusable validators for type checking, range validation,
//! pattern matching, and custom validation logic.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::nodes::node::{Node, Numeric};
use crate::validation::schema::SchemaType;

/// Result of a validation operation
pub type ValidationResult = Result<(), String>;

/// Trait for validators that can check nodes against rules
pub trait Validator {
    /// Validate a node and return Ok(()) or an error message
    fn validate(&self, node: &Node) -> ValidationResult;

    /// Get a description of what this validator checks
    fn description(&self) -> String;
}

/// Validates node type matches expected type
#[derive(Debug, Clone)]
pub struct TypeValidator {
    expected_type: SchemaType,
}

impl TypeValidator {
    pub fn new(expected_type: SchemaType) -> Self {
        Self { expected_type }
    }
}

impl Validator for TypeValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        let matches = match (&self.expected_type, node) {
            (SchemaType::String, Node::Str(_, _, _)) => true,
            (SchemaType::Number, Node::Number(_)) => true,
            (SchemaType::Integer, Node::Number(Numeric::Integer(_))) => true,
            (SchemaType::Integer, Node::Number(Numeric::Int32(_))) => true,
            (SchemaType::Integer, Node::Number(Numeric::Int16(_))) => true,
            (SchemaType::Integer, Node::Number(Numeric::Int8(_))) => true,
            (SchemaType::Float, Node::Number(Numeric::Float(_))) => true,
            (SchemaType::Boolean, Node::Boolean(_)) => true,
            (SchemaType::Null, Node::None) => true,
            (SchemaType::Array, Node::Array(_)) => true,
            (SchemaType::Array, Node::Set(_)) => true,
            (SchemaType::Object, Node::Mapping(_)) => true,
            (SchemaType::Any, _) => true,
            _ => false,
        };

        if matches {
            Ok(())
        } else {
            Err(format!(
                "Type mismatch: expected {:?}, got {:?}",
                self.expected_type,
                node_type_name(node)
            ))
        }
    }

    fn description(&self) -> String {
        format!("Type must be {:?}", self.expected_type)
    }
}

/// Validates numeric values are within a range
#[derive(Debug, Clone)]
pub struct RangeValidator {
    min: Option<f64>,
    max: Option<f64>,
}

impl RangeValidator {
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }
}

impl Validator for RangeValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        let value = match node {
            Node::Number(Numeric::Integer(i)) => *i as f64,
            Node::Number(Numeric::Float(f)) => *f,
            Node::Number(Numeric::UInteger(u)) => *u as f64,
            Node::Number(Numeric::Int32(i)) => *i as f64,
            Node::Number(Numeric::UInt32(u)) => *u as f64,
            Node::Number(Numeric::Int16(i)) => *i as f64,
            Node::Number(Numeric::UInt16(u)) => *u as f64,
            Node::Number(Numeric::Int8(i)) => *i as f64,
            Node::Number(Numeric::Byte(b)) => *b as f64,
            _ => return Err("RangeValidator only applies to numeric values".to_string()),
        };

        if let Some(min) = self.min {
            if value < min {
                return Err(format!("Value {} is less than minimum {}", value, min));
            }
        }

        if let Some(max) = self.max {
            if value > max {
                return Err(format!("Value {} is greater than maximum {}", value, max));
            }
        }

        Ok(())
    }

    fn description(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("Value must be between {} and {}", min, max),
            (Some(min), None) => format!("Value must be at least {}", min),
            (None, Some(max)) => format!("Value must be at most {}", max),
            (None, None) => "No range restriction".to_string(),
        }
    }
}

/// Validates string/array length
#[derive(Debug, Clone)]
pub struct LengthValidator {
    min: Option<usize>,
    max: Option<usize>,
}

impl LengthValidator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }
}

impl Validator for LengthValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        let length = match node {
            Node::Str(s, _, _) => s.len(),
            Node::Array(arr) => arr.len(),
            Node::Set(set) => set.len(),
            _ => return Err("LengthValidator only applies to strings and arrays".to_string()),
        };

        if let Some(min) = self.min {
            if length < min {
                return Err(format!("Length {} is less than minimum {}", length, min));
            }
        }

        if let Some(max) = self.max {
            if length > max {
                return Err(format!("Length {} is greater than maximum {}", length, max));
            }
        }

        Ok(())
    }

    fn description(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("Length must be between {} and {}", min, max),
            (Some(min), None) => format!("Length must be at least {}", min),
            (None, Some(max)) => format!("Length must be at most {}", max),
            (None, None) => "No length restriction".to_string(),
        }
    }
}

/// Validates string matches a pattern
#[derive(Debug, Clone)]
pub struct PatternValidator {
    pattern: String,
}

impl PatternValidator {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }

    /// Check if string contains pattern (simple substring match)
    fn matches(&self, s: &str) -> bool {
        s.contains(&self.pattern)
    }
}

impl Validator for PatternValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        match node {
            Node::Str(s, _, _) => {
                if self.matches(s) {
                    Ok(())
                } else {
                    Err(format!("String does not match pattern '{}'", self.pattern))
                }
            }
            _ => Err("PatternValidator only applies to strings".to_string()),
        }
    }

    fn description(&self) -> String {
        format!("Must match pattern: {}", self.pattern)
    }
}

/// Validates value is one of allowed enum values
#[derive(Debug, Clone)]
pub struct EnumValidator {
    allowed: Vec<String>,
}

impl EnumValidator {
    pub fn new(allowed: Vec<String>) -> Self {
        Self { allowed }
    }
}

impl Validator for EnumValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        let value = match node {
            Node::Str(s, _, _) => s.as_str(),
            _ => return Err("EnumValidator only applies to strings".to_string()),
        };

        if self.allowed.contains(&value.to_string()) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' is not one of: {}",
                value,
                self.allowed.join(", ")
            ))
        }
    }

    fn description(&self) -> String {
        format!("Must be one of: {}", self.allowed.join(", "))
    }
}

/// Validates a required field exists
#[derive(Debug, Clone)]
pub struct RequiredValidator {
    field_name: String,
}

impl RequiredValidator {
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
        }
    }
}

impl Validator for RequiredValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        match node {
            Node::Mapping(pairs) => {
                let found = pairs.iter().any(|(k, _)| match k {
                    Node::Str(s, _, _) => s == &self.field_name,
                    _ => false,
                });

                if found {
                    Ok(())
                } else {
                    Err(format!("Required field '{}' is missing", self.field_name))
                }
            }
            _ => Err("RequiredValidator only applies to mappings".to_string()),
        }
    }

    fn description(&self) -> String {
        format!("Field '{}' is required", self.field_name)
    }
}

/// Custom validator using a closure
pub struct CustomValidator {
    validate_fn: Box<dyn Fn(&Node) -> ValidationResult>,
    description: String,
}

impl CustomValidator {
    pub fn new<F>(description: impl Into<String>, validate_fn: F) -> Self
    where
        F: Fn(&Node) -> ValidationResult + 'static,
    {
        Self {
            validate_fn: Box::new(validate_fn),
            description: description.into(),
        }
    }
}

impl Validator for CustomValidator {
    fn validate(&self, node: &Node) -> ValidationResult {
        (self.validate_fn)(node)
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Get human-readable name for node type
fn node_type_name(node: &Node) -> &'static str {
    match node {
        Node::Boolean(_) => "Boolean",
        Node::Number(_) => "Number",
        Node::Str(_, _, _) => "String",
        Node::Array(_) => "Array",
        Node::Set(_) => "Set",
        Node::Mapping(_) => "Mapping",
        Node::Comment(_) => "Comment",
        Node::Document(_) => "Document",
        Node::Anchored(_, _) => "Anchored",
        Node::Tagged(_, _) => "Tagged",
        Node::Alias(_) => "Alias",
        Node::Documents(_) => "Documents",
        Node::None => "Null",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_validator() {
        let validator = TypeValidator::new(SchemaType::String);

        assert!(validator.validate(&Node::from("hello")).is_ok());
        assert!(validator.validate(&Node::from(42)).is_err());
    }

    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::new(Some(0.0), Some(100.0));

        assert!(
            validator
                .validate(&Node::Number(Numeric::Integer(50)))
                .is_ok()
        );
        assert!(
            validator
                .validate(&Node::Number(Numeric::Integer(150)))
                .is_err()
        );
        assert!(
            validator
                .validate(&Node::Number(Numeric::Integer(-10)))
                .is_err()
        );
    }

    #[test]
    fn test_length_validator() {
        let validator = LengthValidator::new(Some(3), Some(10));

        assert!(validator.validate(&Node::from("hello")).is_ok());
        assert!(validator.validate(&Node::from("hi")).is_err());
        assert!(validator.validate(&Node::from("verylongstring")).is_err());
    }

    #[test]
    fn test_pattern_validator() {
        let validator = PatternValidator::new("@");

        assert!(validator.validate(&Node::from("user@example.com")).is_ok());
        assert!(validator.validate(&Node::from("invalid")).is_err());
    }

    #[test]
    fn test_enum_validator() {
        let validator = EnumValidator::new(vec![
            "red".to_string(),
            "green".to_string(),
            "blue".to_string(),
        ]);

        assert!(validator.validate(&Node::from("red")).is_ok());
        assert!(validator.validate(&Node::from("yellow")).is_err());
    }

    #[test]
    fn test_required_validator() {
        let validator = RequiredValidator::new("name");

        let mapping = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::from(30)),
        ]);

        assert!(validator.validate(&mapping).is_ok());

        let mapping2 = Node::Mapping(vec![(Node::from("age"), Node::from(30))]);

        assert!(validator.validate(&mapping2).is_err());
    }

    #[test]
    fn test_custom_validator() {
        let validator = CustomValidator::new("Must be positive", |node| match node {
            Node::Number(Numeric::Integer(i)) if *i > 0 => Ok(()),
            Node::Number(Numeric::Integer(_)) => Err("Number must be positive".to_string()),
            _ => Err("Not a number".to_string()),
        });

        assert!(
            validator
                .validate(&Node::Number(Numeric::Integer(10)))
                .is_ok()
        );
        assert!(
            validator
                .validate(&Node::Number(Numeric::Integer(-5)))
                .is_err()
        );
    }
}
