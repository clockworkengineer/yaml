/// Core helpers for building validation errors (DRY for validators)
pub struct ValidationContextCore;

impl ValidationContextCore {
    pub fn fail_type_mismatch(expected: &SchemaType, node: &Node) -> ValidationError {
        ValidationError::TypeMismatch {
            expected: format!("{:?}", expected),
            found: crate::validation::validators::node_type_name(node).to_string(),
        }
    }

    pub fn fail_range(value: f64, min: Option<f64>, max: Option<f64>) -> ValidationError {
        ValidationError::RangeError { value, min, max }
    }

    pub fn fail_required(field: &str) -> ValidationError {
        ValidationError::RequiredFieldMissing {
            field: field.to_string(),
        }
    }
}
/// Validation engine for executing schema validation against YAML nodes
///
/// Provides SchemaValidator that traverses nodes and applies validation rules.
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use log::warn;

use crate::nodes::node::{Node, Numeric};
use crate::validation::error::ValidationError;
use crate::validation::schema::{PropertySchema, Schema, SchemaType};
use crate::validation::validators::{
    EnumValidator, LengthValidator, PatternValidator, RangeValidator, TypeValidator, Validator,
};

/// Context for tracking validation state during traversal
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Current path in document
    path: Vec<String>,
    /// Accumulated errors
    errors: Vec<ValidationError>,
    /// Whether to stop on first error
    fail_fast: bool,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            errors: Vec::new(),
            fail_fast: false,
        }
    }

    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Add a path segment
    fn push(&mut self, segment: impl Into<String>) {
        self.path.push(segment.into());
    }

    /// Remove last path segment
    fn pop(&mut self) {
        self.path.pop();
    }

    /// Get current path as string
    #[allow(dead_code)]
    fn current_path(&self) -> String {
        if self.path.is_empty() {
            "$".to_string()
        } else {
            format!("$.{}", self.path.join("."))
        }
    }

    /// Record an error and log it
    fn add_error(&mut self, error: ValidationError) {
        warn!("Validation error: {:?}", error);
        self.errors.push(error);
    }

    /// Check if we should stop validation
    fn should_stop(&self) -> bool {
        self.fail_fast && !self.errors.is_empty()
    }

    /// Get all errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Check if validation succeeded
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main validator for executing schema validation
pub struct SchemaValidator {
    schema: Schema,
}

impl SchemaValidator {
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    /// Validate a node against the schema
    pub fn validate(&self, node: &Node) -> Result<(), Vec<ValidationError>> {
        let mut ctx = ValidationContext::new();
        self.validate_with_context(node, &mut ctx);

        if ctx.is_valid() {
            Ok(())
        } else {
            Err(ctx.errors)
        }
    }

    /// Validate with custom context
    pub fn validate_with_context(&self, node: &Node, ctx: &mut ValidationContext) {
        self.validate_property(node, &self.schema.root, ctx);
    }

    /// Validate node against property schema
    fn validate_property(&self, node: &Node, schema: &PropertySchema, ctx: &mut ValidationContext) {
        if ctx.should_stop() {
            return;
        }

        // Type validation
        let type_validator = TypeValidator::new(schema.schema_type.clone());
        if let Err(err) = type_validator.validate(node) {
            ctx.add_error(err);
            return;
        }

        // Range validation for numbers
        if let (Some(min), Some(max)) = (schema.minimum, schema.maximum) {
            let validator = RangeValidator::new(Some(min), Some(max));
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        } else if let Some(min) = schema.minimum {
            let validator = RangeValidator::new(Some(min), None);
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        } else if let Some(max) = schema.maximum {
            let validator = RangeValidator::new(None, Some(max));
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        }

        // Length validation for strings/arrays
        if let (Some(min), Some(max)) = (schema.min_length, schema.max_length) {
            let validator = LengthValidator::new(Some(min), Some(max));
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        } else if let Some(min) = schema.min_length {
            let validator = LengthValidator::new(Some(min), None);
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        } else if let Some(max) = schema.max_length {
            let validator = LengthValidator::new(None, Some(max));
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        }

        // Pattern validation
        if let Some(ref pattern) = schema.pattern {
            let validator = PatternValidator::new(pattern.clone());
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        }

        // Enum validation
        if let Some(ref allowed) = schema.enum_values {
            let validator = EnumValidator::new(allowed.clone());
            if let Err(err) = validator.validate(node) {
                ctx.add_error(err);
                return;
            }
        }

        // Validate nested structures
        match (&schema.schema_type, node) {
            (SchemaType::Array, Node::Array(arr)) => {
                if let Some(ref items) = schema.items {
                    // Validate each item against the schema
                    for (i, item) in arr.iter().enumerate() {
                        ctx.push(format!("[{}]", i));
                        self.validate_property(item, items, ctx);
                        ctx.pop();

                        if ctx.should_stop() {
                            return;
                        }
                    }
                }
            }
            (SchemaType::Object, Node::Mapping(pairs)) => {
                if let Some(ref properties) = schema.properties {
                    // Build a map of property names to values
                    let mut props = BTreeMap::new();
                    for (key, value) in pairs {
                        if let Node::Str(k, _, _) = key {
                            props.insert(k.as_str(), value);
                        }
                    }

                    // Validate each property against its schema
                    for (prop_name, prop_schema) in properties {
                        if let Some(value) = props.get(prop_name.as_str()) {
                            ctx.push(prop_name.clone());
                            self.validate_property(value, prop_schema, ctx);
                            ctx.pop();

                            if ctx.should_stop() {
                                return;
                            }
                        } else if prop_schema.required {
                            ctx.add_error(ValidationError::RequiredFieldMissing {
                                field: prop_name.clone(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Get human-readable name for node type
#[allow(dead_code)]
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

/// Convert node to string representation for uniqueness checking
#[allow(dead_code)]
fn node_to_string(node: &Node) -> String {
    match node {
        Node::Boolean(b) => b.to_string(),
        Node::Number(Numeric::Integer(i)) => i.to_string(),
        Node::Number(Numeric::Float(f)) => f.to_string(),
        Node::Number(Numeric::UInteger(u)) => u.to_string(),
        Node::Number(Numeric::Int32(i)) => i.to_string(),
        Node::Number(Numeric::UInt32(u)) => u.to_string(),
        Node::Number(Numeric::Int16(i)) => i.to_string(),
        Node::Number(Numeric::UInt16(u)) => u.to_string(),
        Node::Number(Numeric::Int8(i)) => i.to_string(),
        Node::Number(Numeric::Byte(b)) => b.to_string(),
        Node::Str(s, _, _) => s.clone(),
        Node::None => "null".to_string(),
        _ => format!("{:?}", node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::schema::Schema;

    #[test]
    fn test_simple_validation() {
        let schema = Schema::string();
        let validator = SchemaValidator::new(schema);

        assert!(validator.validate(&Node::from("hello")).is_ok());
        assert!(validator.validate(&Node::from(42)).is_err());
    }

    #[test]
    fn test_range_validation() {
        let schema = Schema {
            root: PropertySchema::new(SchemaType::Integer)
                .with_minimum(0.0)
                .with_maximum(100.0),
            title: None,
            description: None,
        };
        let validator = SchemaValidator::new(schema);

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
    }

    #[test]
    fn test_object_validation() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "name".to_string(),
            PropertySchema::new(SchemaType::String).required(),
        );
        properties.insert("age".to_string(), PropertySchema::new(SchemaType::Integer));

        let schema = Schema::object(properties);
        let validator = SchemaValidator::new(schema);

        let valid_obj = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::Number(Numeric::Integer(30))),
        ]);
        assert!(validator.validate(&valid_obj).is_ok());

        let invalid_obj = Node::Mapping(vec![(
            Node::from("age"),
            Node::Number(Numeric::Integer(30)),
        )]);
        assert!(validator.validate(&invalid_obj).is_err());
    }

    #[test]
    fn test_array_validation() {
        let schema = Schema::array(PropertySchema::new(SchemaType::Integer));
        let validator = SchemaValidator::new(schema);

        let valid_arr = Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
        ]);
        assert!(validator.validate(&valid_arr).is_ok());

        let wrong_type = Node::Array(vec![Node::from("not a number")]);
        assert!(validator.validate(&wrong_type).is_err());
    }

    #[test]
    fn test_validation_error_paths() {
        let mut user_props = BTreeMap::new();
        user_props.insert("name".to_string(), PropertySchema::new(SchemaType::String));
        user_props.insert("age".to_string(), PropertySchema::new(SchemaType::Integer));

        let mut root_props = BTreeMap::new();
        root_props.insert(
            "user".to_string(),
            PropertySchema::new(SchemaType::Object).with_properties(user_props),
        );

        let schema = Schema::object(root_props);
        let validator = SchemaValidator::new(schema);

        let obj = Node::Mapping(vec![(
            Node::from("user"),
            Node::Mapping(vec![
                (Node::from("name"), Node::from("Alice")),
                (Node::from("age"), Node::from("not a number")),
            ]),
        )]);

        let result = validator.validate(&obj);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        // No path field anymore; just check error kind
        let found_required = errors
            .iter()
            .any(|e| matches!(e, ValidationError::TypeMismatch { .. }));
        assert!(found_required);
    }
}
