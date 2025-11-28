//! Error types for validation

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    TypeMismatch {
        expected: String,
        found: String,
    },
    RangeError {
        value: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
    LengthError {
        length: usize,
        min: Option<usize>,
        max: Option<usize>,
    },
    PatternMismatch {
        pattern: String,
        value: String,
    },
    EnumMismatch {
        allowed: Vec<String>,
        value: String,
    },
    RequiredFieldMissing {
        field: String,
    },
    InvalidNodeType {
        validator: String,
        found: String,
    },
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, found)
            }
            ValidationError::RangeError { value, min, max } => {
                write!(f, "Value {} out of range [{:?}, {:?}]", value, min, max)
            }
            ValidationError::LengthError { length, min, max } => {
                write!(f, "Length {} out of range [{:?}, {:?}]", length, min, max)
            }
            ValidationError::PatternMismatch { pattern, value } => {
                write!(f, "Value '{}' does not match pattern '{}'", value, pattern)
            }
            ValidationError::EnumMismatch { allowed, value } => {
                write!(f, "Value '{}' is not one of: {}", value, allowed.join(", "))
            }
            ValidationError::RequiredFieldMissing { field } => {
                write!(f, "Required field '{}' is missing", field)
            }
            ValidationError::InvalidNodeType { validator, found } => {
                write!(f, "{} only applies to {}", validator, found)
            }
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}
