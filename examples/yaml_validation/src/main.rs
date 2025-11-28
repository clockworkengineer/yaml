//! YAML Validation Example
//!
//! Demonstrates JSON Schema-style validation for YAML documents.

use std::collections::BTreeMap;
use yaml_lib::*;
use yaml_lib::validation::error::ValidationError;

/// Helper to parse YAML from a string and get the first document
fn parse_yaml(yaml: &str) -> Node {
    let mut source = BufferSource::new(yaml.as_bytes());
    let parsed = parse(&mut source).unwrap();

    // Extract first document from Documents wrapper
    match parsed {
        Node::Documents(docs) => {
            if let Some(Node::Document(ref doc)) = docs.get(0) {
                if !doc.is_empty() {
                    doc[0].clone()
                } else {
                    Node::None
                }
            } else {
                Node::None
            }
        }
        _ => parsed,
    }
}

fn main() {
    println!("========================================");
    println!("YAML Validation Examples");
    println!("========================================\n");

    example_1_simple_type_validation();
    example_2_object_with_required_fields();
    example_3_array_validation();
    example_4_nested_object_validation();
    example_5_range_and_length_constraints();
    example_6_pattern_and_enum_validation();
    example_7_custom_validators();
    example_8_real_world_config_validation();
}

/// Example 1: Simple type validation
fn example_1_simple_type_validation() {
    println!("Example 1: Simple Type Validation");
    println!("-----------------------------------");

    let schema = Schema::string();
    let validator = SchemaValidator::new(schema);

    // Valid string
    let message_node = Node::from("Hello, World!");
    match validator.validate(&message_node) {
        Ok(_) => println!("✓ Valid string accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid: number instead of string
    let count_node = Node::from(42);
    match validator.validate(&count_node) {
        Ok(_) => println!("✗ Should have rejected number"),
        Err(errors) => println!("✓ Correctly rejected number: {}", errors[0].message),
    }

    println!();
}

/// Example 2: Object with required fields
fn example_2_object_with_required_fields() {
    println!("Example 2: Object with Required Fields");
    println!("---------------------------------------");

    // Define schema: { name: string (required), age: integer (optional) }
    let mut properties = BTreeMap::new();
    properties.insert(
        "name".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_min_length(2)
            .with_max_length(50),
    );
    properties.insert(
        "age".to_string(),
        PropertySchema::new(SchemaType::Integer)
            .with_minimum(0.0)
            .with_maximum(150.0),
    );

    let schema = Schema::object(properties);
    let validator = SchemaValidator::new(schema);

    // Valid: has required name
    let valid_yaml = r#"
name: Alice
age: 30
"#;
    let valid_doc = parse_yaml(valid_yaml);
    match validator.validate(&valid_doc) {
        Ok(_) => println!("✓ Valid user object accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid: missing required name
    let invalid_yaml = r#"
age: 25
"#;
    let invalid_doc = parse_yaml(invalid_yaml);
    match validator.validate(&invalid_doc) {
        Ok(_) => println!("✗ Should have rejected missing required field"),
        Err(errors) => println!("✓ Correctly rejected: {}", errors[0].message),
    }

    println!();
}

/// Example 3: Array validation
fn example_3_array_validation() {
    println!("Example 3: Array Validation");
    println!("----------------------------");

    // Array of integers
    let schema = Schema::array(PropertySchema::new(SchemaType::Integer));
    let validator = SchemaValidator::new(schema);

    // Valid array
    let valid_yaml = "numbers: [1, 2, 3, 4, 5]";
    let valid_doc = parse_yaml(valid_yaml);
    match validator.validate(&valid_doc["numbers"]) {
        Ok(_) => println!("✓ Valid integer array accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid: mixed types
    let invalid_yaml = "mixed: [1, two, 3]";
    let invalid_doc = parse_yaml(invalid_yaml);
    match validator.validate(&invalid_doc["mixed"]) {
        Ok(_) => println!("✗ Should have rejected mixed types"),
        Err(errors) => println!("✓ Correctly rejected mixed types: {}", errors[0].message),
    }

    println!();
}

/// Example 4: Nested object validation
fn example_4_nested_object_validation() {
    println!("Example 4: Nested Object Validation");
    println!("------------------------------------");

    // Define nested user schema
    let mut address_props = BTreeMap::new();
    address_props.insert(
        "street".to_string(),
        PropertySchema::new(SchemaType::String).required(),
    );
    address_props.insert(
        "city".to_string(),
        PropertySchema::new(SchemaType::String).required(),
    );
    address_props.insert(
        "zipcode".to_string(),
        PropertySchema::new(SchemaType::String)
            .with_pattern("-")
            .with_min_length(5),
    );

    let mut user_props = BTreeMap::new();
    user_props.insert(
        "name".to_string(),
        PropertySchema::new(SchemaType::String).required(),
    );
    user_props.insert(
        "address".to_string(),
        PropertySchema::new(SchemaType::Object)
            .with_properties(address_props)
            .required(),
    );

    let schema = Schema::object(user_props);
    let validator = SchemaValidator::new(schema);

    // Valid nested structure
    let valid_yaml = r#"
name: Bob
address:
  street: 123 Main St
  city: Springfield
  zipcode: 12345-6789
"#;
    let valid_doc = parse_yaml(valid_yaml);
    match validator.validate(&valid_doc) {
        Ok(_) => println!("✓ Valid nested object accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid: missing nested required field
    let invalid_yaml = r#"
name: Charlie
address:
  city: Springfield
"#;
    let invalid_doc = parse_yaml(invalid_yaml);
    match validator.validate(&invalid_doc) {
        Ok(_) => println!("✗ Should have rejected missing nested field"),
        Err(errors) => {
            println!("✓ Correctly rejected: {}", errors[0].message);
            println!("  Path: {}", errors[0].path);
        }
    }

    println!();
}

/// Example 5: Range and length constraints
fn example_5_range_and_length_constraints() {
    println!("Example 5: Range and Length Constraints");
    println!("----------------------------------------");

    // Password schema: 8-20 characters
    let mut props = BTreeMap::new();
    props.insert(
        "username".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_min_length(3)
            .with_max_length(20),
    );
    props.insert(
        "password".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_min_length(8)
            .with_max_length(50),
    );
    props.insert(
        "age".to_string(),
        PropertySchema::new(SchemaType::Integer)
            .with_minimum(18.0)
            .with_maximum(100.0),
    );

    let schema = Schema::object(props);
    let validator = SchemaValidator::new(schema);

    // Valid credentials
    let valid_yaml = r#"
username: alice123
password: supersecret123
age: 25
"#;
    let valid_doc = parse_yaml(valid_yaml);
    match validator.validate(&valid_doc) {
        Ok(_) => println!("✓ Valid credentials accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid: password too short
    let invalid_yaml = r#"
username: bob
password: short
age: 30
"#;
    let invalid_doc = parse_yaml(invalid_yaml);
    match validator.validate(&invalid_doc) {
        Ok(_) => println!("✗ Should have rejected short password"),
        Err(errors) => println!("✓ Correctly rejected: {}", errors[0].message),
    }

    // Invalid: age out of range
    let invalid_age_yaml = r#"
username: charlie
password: validpassword123
age: 15
"#;
    let invalid_age_doc = parse_yaml(invalid_age_yaml);
    match validator.validate(&invalid_age_doc) {
        Ok(_) => println!("✗ Should have rejected underage"),
        Err(errors) => println!("✓ Correctly rejected age: {}", errors[0].message),
    }

    println!();
}

/// Example 6: Pattern and enum validation
fn example_6_pattern_and_enum_validation() {
    println!("Example 6: Pattern and Enum Validation");
    println!("---------------------------------------");

    let mut props = BTreeMap::new();
    props.insert(
        "email".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_pattern("@"), // Simple email check
    );
    props.insert(
        "role".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_enum(vec![
                "admin".to_string(),
                "user".to_string(),
                "guest".to_string(),
            ]),
    );

    let schema = Schema::object(props);
    let validator = SchemaValidator::new(schema);

    // Valid
    let valid_yaml = r#"
email: user@example.com
role: admin
"#;
    let valid_doc = parse_yaml(valid_yaml);
    match validator.validate(&valid_doc) {
        Ok(_) => println!("✓ Valid email and role accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid: bad email
    let invalid_email_yaml = r#"
email: notanemail
role: user
"#;
    let invalid_email_doc = parse_yaml(invalid_email_yaml);
    match validator.validate(&invalid_email_doc) {
        Ok(_) => println!("✗ Should have rejected invalid email"),
        Err(errors) => println!("✓ Correctly rejected email: {}", errors[0].message),
    }

    // Invalid: bad role
    let invalid_role_yaml = r#"
email: test@example.com
role: superadmin
"#;
    let invalid_role_doc = parse_yaml(invalid_role_yaml);
    match validator.validate(&invalid_role_doc) {
        Ok(_) => println!("✗ Should have rejected invalid role"),
        Err(errors) => println!("✓ Correctly rejected role: {}", errors[0].message),
    }

    println!();
}

/// Example 7: Custom validators
fn example_7_custom_validators() {
    println!("Example 7: Custom Validators");
    println!("-----------------------------");

    // Custom validator: port number must be > 1024
    let custom_validator =
        CustomValidator::new("Port must be greater than 1024", |node| match node {
            Node::Number(Numeric::Integer(port)) if *port > 1024 => Ok(()),
            Node::Number(Numeric::Integer(port)) => Err(ValidationError::Custom(format!(
                "Port {} is reserved (must be > 1024)",
                port
            ))),
            _ => Err(ValidationError::Custom(
                "Port must be an integer".to_string(),
            )),
        });

    // Test valid port
    let valid_port = Node::Number(Numeric::Integer(8080));
    match custom_validator.validate(&valid_port) {
        Ok(_) => println!("✓ Valid port 8080 accepted"),
        Err(e) => println!("✗ Unexpected error: {}", e),
    }

    // Test invalid port
    let invalid_port = Node::Number(Numeric::Integer(80));
    match custom_validator.validate(&invalid_port) {
        Ok(_) => println!("✗ Should have rejected port 80"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    println!();
}

/// Example 8: Real-world configuration validation
fn example_8_real_world_config_validation() {
    println!("Example 8: Real-World Configuration Validation");
    println!("-----------------------------------------------");

    // Define server config schema
    let mut server_props = BTreeMap::new();
    server_props.insert(
        "host".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_description("Server hostname"),
    );
    server_props.insert(
        "port".to_string(),
        PropertySchema::new(SchemaType::Integer)
            .required()
            .with_minimum(1024.0)
            .with_maximum(65535.0),
    );
    server_props.insert(
        "timeout".to_string(),
        PropertySchema::new(SchemaType::Integer)
            .with_minimum(0.0)
            .with_maximum(300.0),
    );

    let mut db_props = BTreeMap::new();
    db_props.insert(
        "driver".to_string(),
        PropertySchema::new(SchemaType::String)
            .required()
            .with_enum(vec![
                "postgres".to_string(),
                "mysql".to_string(),
                "sqlite".to_string(),
            ]),
    );
    db_props.insert(
        "host".to_string(),
        PropertySchema::new(SchemaType::String).required(),
    );
    db_props.insert(
        "port".to_string(),
        PropertySchema::new(SchemaType::Integer).required(),
    );
    db_props.insert(
        "database".to_string(),
        PropertySchema::new(SchemaType::String).required(),
    );

    let mut config_props = BTreeMap::new();
    config_props.insert(
        "server".to_string(),
        PropertySchema::new(SchemaType::Object)
            .with_properties(server_props)
            .required(),
    );
    config_props.insert(
        "database".to_string(),
        PropertySchema::new(SchemaType::Object)
            .with_properties(db_props)
            .required(),
    );
    config_props.insert(
        "features".to_string(),
        PropertySchema::new(SchemaType::Array).with_items(PropertySchema::new(SchemaType::String)),
    );

    let schema = Schema::new(PropertySchema::new(SchemaType::Object).with_properties(config_props))
        .with_title("Application Configuration")
        .with_description("Configuration schema for the application");

    let validator = SchemaValidator::new(schema);

    // Valid configuration
    let valid_config = r#"
server:
  host: localhost
  port: 8080
  timeout: 30
database:
  driver: postgres
  host: db.example.com
  port: 5432
  database: myapp
features:
  - auth
  - logging
  - metrics
"#;
    let valid_doc = parse_yaml(valid_config);
    match validator.validate(&valid_doc) {
        Ok(_) => println!("✓ Valid configuration accepted"),
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    // Invalid configuration: missing required database.driver
    let invalid_config = r#"
server:
  host: localhost
  port: 8080
database:
  host: db.example.com
  port: 5432
  database: myapp
"#;
    let invalid_doc = parse_yaml(invalid_config);
    match validator.validate(&invalid_doc) {
        Ok(_) => println!("✗ Should have rejected incomplete config"),
        Err(errors) => {
            println!("✓ Found {} validation error(s):", errors.len());
            for error in errors {
                println!("  - {}: {}", error.path, error.message);
            }
        }
    }

    // Invalid configuration: bad port number
    let invalid_port_config = r#"
server:
  host: localhost
  port: 99999
database:
  driver: postgres
  host: db.example.com
  port: 5432
  database: myapp
"#;
    let invalid_port_doc = parse_yaml(invalid_port_config);
    match validator.validate(&invalid_port_doc) {
        Ok(_) => println!("✗ Should have rejected invalid port"),
        Err(errors) => {
            println!("✓ Correctly rejected invalid port:");
            println!("  - {}: {}", errors[0].path, errors[0].message);
        }
    }

    println!();
    println!("========================================");
    println!("All validation examples completed!");
    println!("========================================");
}
