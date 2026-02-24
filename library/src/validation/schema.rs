//! Schema Definition Types for YAML Validation
//!
//! This module provides types and structures for defining the expected schema of YAML documents,
//! similar to JSON Schema. It enables specifying types, constraints, and validation rules for YAML data.
//!
//! # Features
//! - Schema types for YAML nodes
//! - Support for constraints and validation rules
//! - Enables schema-driven validation
//!
//! # Usage
//! Use these types to define and enforce the structure of YAML documents.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Schema type specifying the expected YAML node type
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    /// String value
    String,
    /// Numeric value (integer or float)
    Number,
    /// Integer value only
    Integer,
    /// Float value only
    Float,
    /// Boolean value
    Boolean,
    /// Null value
    Null,
    /// Array/sequence
    Array,
    /// Object/mapping
    Object,
    /// Any type allowed
    Any,
}

/// Schema for object/mapping properties
#[derive(Debug, Clone)]
pub struct PropertySchema {
    /// Type of this property
    pub schema_type: SchemaType,
    /// Whether this property is required
    pub required: bool,
    /// Description of this property
    pub description: Option<String>,
    /// Minimum value (for numbers)
    pub minimum: Option<f64>,
    /// Maximum value (for numbers)
    pub maximum: Option<f64>,
    /// Minimum length (for strings/arrays)
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    pub max_length: Option<usize>,
    /// Pattern to match (for strings)
    pub pattern: Option<String>,
    /// Enum of allowed values (for strings)
    pub enum_values: Option<Vec<String>>,
    /// Nested schema for objects
    pub properties: Option<BTreeMap<String, PropertySchema>>,
    /// Schema for array items
    pub items: Option<Box<PropertySchema>>,
    /// Default value
    pub default: Option<String>,
}

impl PropertySchema {
    /// Create a new property schema with the given type
    pub fn new(schema_type: SchemaType) -> Self {
        Self {
            schema_type,
            required: false,
            description: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            properties: None,
            items: None,
            default: None,
        }
    }

    /// Mark this property as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set minimum value
    pub fn with_minimum(mut self, min: f64) -> Self {
        self.minimum = Some(min);
        self
    }

    /// Set maximum value
    pub fn with_maximum(mut self, max: f64) -> Self {
        self.maximum = Some(max);
        self
    }

    /// Set minimum length
    pub fn with_min_length(mut self, len: usize) -> Self {
        self.min_length = Some(len);
        self
    }

    /// Set maximum length
    pub fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = Some(len);
        self
    }

    /// Set pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set enum values
    pub fn with_enum(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Set object properties
    pub fn with_properties(mut self, props: BTreeMap<String, PropertySchema>) -> Self {
        self.properties = Some(props);
        self
    }

    /// Set array items schema
    pub fn with_items(mut self, items: PropertySchema) -> Self {
        self.items = Some(Box::new(items));
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// Schema for arrays
#[derive(Debug, Clone)]
pub struct ArraySchema {
    /// Schema for items in the array
    pub items: PropertySchema,
    /// Minimum number of items
    pub min_items: Option<usize>,
    /// Maximum number of items
    pub max_items: Option<usize>,
    /// Whether items must be unique
    pub unique_items: bool,
}

impl ArraySchema {
    /// Create a new array schema
    pub fn new(items: PropertySchema) -> Self {
        Self {
            items,
            min_items: None,
            max_items: None,
            unique_items: false,
        }
    }

    /// Set minimum items
    pub fn with_min_items(mut self, min: usize) -> Self {
        self.min_items = Some(min);
        self
    }

    /// Set maximum items
    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Require unique items
    pub fn with_unique_items(mut self) -> Self {
        self.unique_items = true;
        self
    }
}

/// Schema for objects/mappings
#[derive(Debug, Clone)]
pub struct ObjectSchema {
    /// Properties of this object
    pub properties: BTreeMap<String, PropertySchema>,
    /// Required property names
    pub required: Vec<String>,
    /// Whether additional properties are allowed
    pub additional_properties: bool,
}

impl ObjectSchema {
    /// Create a new object schema
    pub fn new() -> Self {
        Self {
            properties: BTreeMap::new(),
            required: Vec::new(),
            additional_properties: true,
        }
    }

    /// Add a property
    pub fn with_property(mut self, name: impl Into<String>, schema: PropertySchema) -> Self {
        let name_str = name.into();
        if schema.required {
            self.required.push(name_str.clone());
        }
        self.properties.insert(name_str, schema);
        self
    }

    /// Disallow additional properties
    pub fn no_additional_properties(mut self) -> Self {
        self.additional_properties = false;
        self
    }

    /// Add a required property
    pub fn require(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }
}

impl Default for ObjectSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete schema for a YAML document
#[derive(Debug, Clone)]
pub struct Schema {
    /// Root schema type
    pub root: PropertySchema,
    /// Schema title
    pub title: Option<String>,
    /// Schema description
    pub description: Option<String>,
}

impl Schema {
    /// Create a new schema with the given root type
    pub fn new(root: PropertySchema) -> Self {
        Self {
            root,
            title: None,
            description: None,
        }
    }

    /// Set schema title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set schema description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Create a simple string schema
    pub fn string() -> Self {
        Self::new(PropertySchema::new(SchemaType::String))
    }

    /// Create a simple number schema
    pub fn number() -> Self {
        Self::new(PropertySchema::new(SchemaType::Number))
    }

    /// Create a simple integer schema
    pub fn integer() -> Self {
        Self::new(PropertySchema::new(SchemaType::Integer))
    }

    /// Create a simple boolean schema
    pub fn boolean() -> Self {
        Self::new(PropertySchema::new(SchemaType::Boolean))
    }

    /// Create an array schema
    pub fn array(items: PropertySchema) -> Self {
        Self::new(PropertySchema::new(SchemaType::Array).with_items(items))
    }

    /// Create an object schema
    pub fn object(properties: BTreeMap<String, PropertySchema>) -> Self {
        Self::new(PropertySchema::new(SchemaType::Object).with_properties(properties))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_schema_builder() {
        let schema = PropertySchema::new(SchemaType::String)
            .required()
            .with_min_length(5)
            .with_max_length(50)
            .with_description("A test string");

        assert_eq!(schema.schema_type, SchemaType::String);
        assert!(schema.required);
        assert_eq!(schema.min_length, Some(5));
        assert_eq!(schema.max_length, Some(50));
        assert_eq!(schema.description, Some("A test string".to_string()));
    }

    #[test]
    fn test_array_schema() {
        let array = ArraySchema::new(PropertySchema::new(SchemaType::Integer))
            .with_min_items(1)
            .with_max_items(10)
            .with_unique_items();

        assert_eq!(array.items.schema_type, SchemaType::Integer);
        assert_eq!(array.min_items, Some(1));
        assert_eq!(array.max_items, Some(10));
        assert!(array.unique_items);
    }

    #[test]
    fn test_object_schema() {
        let obj = ObjectSchema::new()
            .with_property("name", PropertySchema::new(SchemaType::String).required())
            .with_property("age", PropertySchema::new(SchemaType::Integer))
            .no_additional_properties();

        assert_eq!(obj.properties.len(), 2);
        assert_eq!(obj.required.len(), 1);
        assert!(!obj.additional_properties);
    }

    #[test]
    fn test_schema_builders() {
        let string_schema = Schema::string();
        assert!(matches!(string_schema.root.schema_type, SchemaType::String));

        let number_schema = Schema::number();
        assert!(matches!(number_schema.root.schema_type, SchemaType::Number));

        let int_schema = Schema::integer();
        assert!(matches!(int_schema.root.schema_type, SchemaType::Integer));
    }

    #[test]
    fn test_schema_with_metadata() {
        let schema = Schema::string()
            .with_title("User Name")
            .with_description("The name of the user");

        assert_eq!(schema.title, Some("User Name".to_string()));
        assert_eq!(schema.description, Some("The name of the user".to_string()));
    }
}
