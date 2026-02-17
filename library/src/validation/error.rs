//! Error types for validation

use alloc::string::String;
use alloc::vec::Vec;

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

/// Fully-formed validation failure combining a path within the document
/// and a concrete ValidationError describing what went wrong at that path.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    /// Path segments from the root to the failing location
    pub path: Vec<String>,
    /// Underlying validation error at that location
    pub error: ValidationError,
}

impl ValidationIssue {
    /// Build a new issue from a path reference and an underlying error.
    pub fn new(path: &[String], error: ValidationError) -> Self {
        Self {
            path: path.to_vec(),
            error,
        }
    }

    /// Human-readable message combining error description and path.
    pub fn message(&self) -> String {
        if self.path.is_empty() {
            self.error.to_string()
        } else {
            format!("{} at {}", self.error, self.path.join("."))
        }
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}
