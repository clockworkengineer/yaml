//! Module: embedded/lightweight_node.rs
//!
//! Lightweight node representation for embedded systems with fixed-size buffers
//! and reduced memory footprint. Uses indices instead of Box pointers and
//! provides bounded collections.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::embedded::config::*;
use crate::nodes::node::{BlockStyle, QuoteType};

/// Lightweight numeric representation for embedded systems
#[derive(Clone, Debug, PartialEq)]
pub enum LightNumeric {
    Integer(i32),
    Float(f32),
    Byte(u8),
    Short(i16),
}

impl From<i32> for LightNumeric {
    fn from(value: i32) -> Self {
        LightNumeric::Integer(value)
    }
}

impl From<f32> for LightNumeric {
    fn from(value: f32) -> Self {
        LightNumeric::Float(value)
    }
}

impl From<u8> for LightNumeric {
    fn from(value: u8) -> Self {
        LightNumeric::Byte(value)
    }
}

impl From<i16> for LightNumeric {
    fn from(value: i16) -> Self {
        LightNumeric::Short(value)
    }
}

/// Fixed-size string for embedded systems
#[derive(Clone, Debug, PartialEq)]
pub struct FixedString {
    data: [u8; 256], // Fixed size buffer
    len: usize,
}

impl FixedString {
    pub fn new() -> Self {
        Self {
            data: [0; 256],
            len: 0,
        }
    }

    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let bytes = s.as_bytes();
        if bytes.len() > 256 {
            return Err("String too long for FixedString");
        }
        let mut data = [0; 256];
        data[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            data,
            len: bytes.len(),
        })
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for FixedString {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightweight node for embedded systems with bounded size
#[derive(Clone, Debug, PartialEq)]
pub enum LightNode {
    /// Boolean value
    Boolean(bool),
    /// Numeric value with reduced precision
    Number(LightNumeric),
    /// String with fixed maximum size
    Str(FixedString, QuoteType, BlockStyle),
    /// Reference to array in arena (index-based)
    ArrayRef(u16),
    /// Reference to mapping in arena (index-based)
    MappingRef(u16),
    /// Null value
    None,
}

impl LightNode {
    /// Creates a boolean node
    pub fn boolean(value: bool) -> Self {
        LightNode::Boolean(value)
    }

    /// Creates an integer node
    pub fn integer(value: i32) -> Self {
        LightNode::Number(LightNumeric::Integer(value))
    }

    /// Creates a float node
    pub fn float(value: f32) -> Self {
        LightNode::Number(LightNumeric::Float(value))
    }

    /// Creates a string node from &str
    pub fn string(s: &str) -> Result<Self, &'static str> {
        Ok(LightNode::Str(
            FixedString::from_str(s)?,
            QuoteType::Unquoted,
            BlockStyle::None,
        ))
    }

    /// Creates a null node
    pub fn null() -> Self {
        LightNode::None
    }
}

/// Arena allocator for nodes in embedded systems
/// Uses fixed-size arrays instead of dynamic allocation
#[derive(Debug)]
pub struct NodeArena {
    #[cfg(feature = "alloc")]
    arrays: Vec<Vec<LightNode>>,
    #[cfg(feature = "alloc")]
    mappings: Vec<Vec<(LightNode, LightNode)>>,
    #[cfg(not(feature = "alloc"))]
    arrays: [Option<[LightNode; 32]>; 16],
    #[cfg(not(feature = "alloc"))]
    mappings: [Option<[(LightNode, LightNode); 32]>; 16],
    array_count: usize,
    mapping_count: usize,
}

impl NodeArena {
    #[cfg(feature = "alloc")]
    pub fn new() -> Self {
        Self {
            arrays: Vec::new(),
            mappings: Vec::new(),
            array_count: 0,
            mapping_count: 0,
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn new() -> Self {
        const NONE_ARRAY: Option<[LightNode; 32]> = None;
        const NONE_MAPPING: Option<[(LightNode, LightNode); 32]> = None;
        Self {
            arrays: [NONE_ARRAY; 16],
            mappings: [NONE_MAPPING; 16],
            array_count: 0,
            mapping_count: 0,
        }
    }

    #[cfg(feature = "alloc")]
    pub fn add_array(&mut self, items: Vec<LightNode>) -> Result<u16, &'static str> {
        if items.len() > MAX_SEQUENCE_ITEMS {
            return Err("Array too large");
        }
        let idx = self.array_count;
        if idx >= u16::MAX as usize {
            return Err("Too many arrays");
        }
        self.arrays.push(items);
        self.array_count += 1;
        Ok(idx as u16)
    }

    #[cfg(feature = "alloc")]
    pub fn add_mapping(&mut self, pairs: Vec<(LightNode, LightNode)>) -> Result<u16, &'static str> {
        if pairs.len() > MAX_MAPPING_PAIRS {
            return Err("Mapping too large");
        }
        let idx = self.mapping_count;
        if idx >= u16::MAX as usize {
            return Err("Too many mappings");
        }
        self.mappings.push(pairs);
        self.mapping_count += 1;
        Ok(idx as u16)
    }

    #[cfg(feature = "alloc")]
    pub fn get_array(&self, idx: u16) -> Option<&Vec<LightNode>> {
        self.arrays.get(idx as usize)
    }

    #[cfg(feature = "alloc")]
    pub fn get_mapping(&self, idx: u16) -> Option<&Vec<(LightNode, LightNode)>> {
        self.mappings.get(idx as usize)
    }
}

#[cfg(feature = "alloc")]
impl Default for NodeArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_numeric_conversions() {
        assert_eq!(LightNumeric::from(42i32), LightNumeric::Integer(42));
        assert_eq!(LightNumeric::from(3.14f32), LightNumeric::Float(3.14));
        assert_eq!(LightNumeric::from(255u8), LightNumeric::Byte(255));
        assert_eq!(LightNumeric::from(1000i16), LightNumeric::Short(1000));
    }

    #[test]
    fn test_fixed_string_creation() {
        let s = FixedString::from_str("Hello").unwrap();
        assert_eq!(s.as_str(), "Hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_fixed_string_too_long() {
        let long_str = "a".repeat(257);
        assert!(FixedString::from_str(&long_str).is_err());
    }

    #[test]
    fn test_fixed_string_empty() {
        let s = FixedString::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.as_str(), "");
    }

    #[test]
    fn test_light_node_creation() {
        let bool_node = LightNode::boolean(true);
        assert_eq!(bool_node, LightNode::Boolean(true));

        let int_node = LightNode::integer(42);
        assert_eq!(int_node, LightNode::Number(LightNumeric::Integer(42)));

        let float_node = LightNode::float(3.14);
        assert_eq!(float_node, LightNode::Number(LightNumeric::Float(3.14)));

        let str_node = LightNode::string("test").unwrap();
        match str_node {
            LightNode::Str(s, _, _) => assert_eq!(s.as_str(), "test"),
            _ => panic!("Expected string node"),
        }

        let null_node = LightNode::null();
        assert_eq!(null_node, LightNode::None);
    }

    #[test]
    fn test_light_node_string_too_long() {
        let long_str = "a".repeat(257);
        assert!(LightNode::string(&long_str).is_err());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_arena_arrays() {
        let mut arena = NodeArena::new();
        let items = alloc::vec![
            LightNode::integer(1),
            LightNode::integer(2),
            LightNode::integer(3),
        ];
        let idx = arena.add_array(items.clone()).unwrap();
        let retrieved = arena.get_array(idx).unwrap();
        assert_eq!(retrieved.len(), 3);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_node_arena_mappings() {
        let mut arena = NodeArena::new();
        let pairs = alloc::vec![
            (LightNode::string("key1").unwrap(), LightNode::integer(1)),
            (LightNode::string("key2").unwrap(), LightNode::integer(2)),
        ];
        let idx = arena.add_mapping(pairs.clone()).unwrap();
        let retrieved = arena.get_mapping(idx).unwrap();
        assert_eq!(retrieved.len(), 2);
    }
}
