//! YAML Node Definitions
//!
//! Defines the core `Node` enum and related traits for representing YAML data structures.
//! Includes conversion, cloning, and utility methods for node manipulation and introspection.
//!
//! Copyright (c) 2026 YAML Library Developers

#[allow(dead_code)]
/// Shared trait for node/string conversion and cloning
pub trait NodeStringConvert {
    /// Convert node to string (lossy)
    fn to_string_lossy(&self) -> alloc::string::String;
    /// Get string value if node is a string
    fn as_str(&self) -> Option<&str>;
    /// Clone node as a string node
    fn clone_as_string(&self) -> Option<Node>;
}

impl NodeStringConvert for Node {
    #[inline]
    fn to_string_lossy(&self) -> alloc::string::String {
        match self {
            Node::Str(s, _, _) => s.clone(),
            Node::Number(n) => n.to_string_lossy(),
            Node::Boolean(b) => b.to_string(),
            Node::None => "null".to_string(),
            _ => format!("{:?}", self),
        }
    }
    #[inline]
    fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s, _, _) => Some(s.as_str()),
            _ => None,
        }
    }
    #[inline]
    fn clone_as_string(&self) -> Option<Node> {
        self.as_str().map(|s| Node::from(s))
    }
}
#[cfg(test)]
use crate::nodes::util::{make_node, make_set};

#[cfg(feature = "std")]
use std::ops::{Index, IndexMut};

#[cfg(not(feature = "std"))]
use core::ops::{Index, IndexMut};

/// Represents different numeric types that can be stored in a YAML node
///
/// For embedded systems, consider using smaller numeric types (i32/f32)
/// to reduce memory footprint. The full enum provides maximum flexibility.
#[derive(Clone, Debug, PartialEq)]
/// Numeric
pub enum Numeric {
    Integer(i64),
    Float(f64),
    UInteger(u64),
    Byte(u8),
    Int32(i32),
    UInt32(u32),
    Int16(i16),
    UInt16(u16),
    Int8(i8),
    UInt8(u8),
}

/// Represents how a string was quoted in the source YAML

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub enum QuoteType {
    Unquoted,
    Single,
    Double,
}

/// Represents block/folded style for YAML string nodes
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub enum BlockStyle {
    None,
    Literal,
    Folded,
}

/// A node in the YAML data structure that can represent different types of values.
///
/// This is the main enum for representing YAML data in memory.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// Represents a boolean value (true/false)
    /// Used for YAML boolean values like true/false, yes/no, on/off
    Boolean(bool),
    /// Represents a numeric value (various integer and float types)
    /// Stores numbers using the most appropriate numeric type from the Numeric enum
    Number(Numeric),
    /// Represents a string value and how it was quoted in the source
    /// Used for text content in YAML including multi-line strings
    Str(alloc::string::String, QuoteType, BlockStyle),
    /// Represents an array of other nodes
    /// Used for YAML sequences/lists where order matters
    Array(alloc::vec::Vec<Node>),
    /// Represents a set of unique nodes
    /// Used for YAML sets where order doesn't matter and duplicates are not allowed
    Set(alloc::vec::Vec<Node>),
    /// Represents a mapping where keys are Nodes (allowing quoted metadata)
    /// Stores an ordered sequence of (key, value) node pairs
    Mapping(alloc::vec::Vec<(Node, Node)>),
    /// Represents a comment
    /// Stores documentation and descriptive text that doesn't affect the data structure
    Comment(alloc::string::String),
    /// Represents a document node
    /// Contains a sequence of top-level nodes making up a YAML document
    Document(alloc::vec::Vec<Node>),
    /// Represents an anchored node: a node with an associated anchor name
    /// Stores the inner node and the anchor name (e.g., &anchor)
    Anchored(alloc::boxed::Box<Node>, alloc::string::String),
    /// Represents a tagged node using YAML tag syntax (e.g., !!str, !mytag)
    /// Stores the inner node and the tag string
    Tagged(alloc::boxed::Box<Node>, alloc::string::String),
    /// Represents an alias node that references a previously anchored node
    /// Stores the anchor name (e.g., *alias)
    Alias(alloc::string::String),
    /// Represents a sequence of documents
    Documents(alloc::vec::Vec<Node>),
    /// Represents a null value or uninitialized node
    /// Used for explicit null values in YAML or missing/undefined values
    None,
}

/// Minimal node for no-alloc environments (embedded only)
#[cfg(not(feature = "alloc"))]
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Boolean(bool),
    Number(Numeric),
    None,
}

/// Implements array-style indexing for Node using integer indices
#[cfg(feature = "alloc")]
impl Index<usize> for Node {
    type Output = Node;

    /// Allows accessing array elements using array[index] syntax
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node::Array(arr) => &arr[index],
            Node::Set(set) => &set[index],
            _ => panic!("Cannot index non-array/set node with integer"),
        }
    }
}

/// Implements mapping-style indexing for Node using string keys
#[cfg(feature = "alloc")]
impl Index<&str> for Node {
    type Output = Node;

    /// Allows accessing mapping properties using mapping["key"] syntax
    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return v;
                        }
                    }
                }
                panic!("No such key exists");
            }
            _ => panic!("Cannot index non-mapping node with string"),
        }
    }
}

/// Implements mutable array-style indexing for Node
#[cfg(feature = "alloc")]
impl IndexMut<usize> for Node {
    /// Allows modifying array elements using array[index] = value syntax
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Node::Array(arr) => &mut arr[index],
            Node::Set(set) => &mut set[index],
            _ => panic!("Cannot index non-array/set node with integer"),
        }
    }
}

/// Implements mutable mapping-style indexing for Node
#[cfg(feature = "alloc")]
impl IndexMut<&str> for Node {
    /// Allows modifying mapping properties using mapping["key"] = value syntax
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs.iter_mut() {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return v;
                        }
                    }
                }
                panic!("No such key exists");
            }
            _ => panic!("Cannot index non-mapping node with string"),
        }
    }
}

/// Converts a vector of values into an array node
#[cfg(feature = "alloc")]
impl<T: Into<Node>> From<Vec<T>> for Node {
    fn from(value: Vec<T>) -> Self {
        Node::Array(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<i64> for Numeric {
    fn from(value: i64) -> Self {
        Numeric::Integer(value)
    }
}

impl From<f64> for Numeric {
    fn from(value: f64) -> Self {
        Numeric::Float(value)
    }
}

impl From<u64> for Numeric {
    fn from(value: u64) -> Self {
        Numeric::UInteger(value)
    }
}

impl From<u8> for Numeric {
    fn from(value: u8) -> Self {
        Numeric::Byte(value)
    }
}

impl From<i32> for Numeric {
    fn from(value: i32) -> Self {
        Numeric::Int32(value)
    }
}

impl From<u32> for Numeric {
    fn from(value: u32) -> Self {
        Numeric::UInt32(value)
    }
}

impl From<i16> for Numeric {
    fn from(value: i16) -> Self {
        Numeric::Int16(value)
    }
}

impl From<u16> for Numeric {
    fn from(value: u16) -> Self {
        Numeric::UInt16(value)
    }
}

impl From<i8> for Numeric {
    fn from(value: i8) -> Self {
        Numeric::Int8(value)
    }
}

#[cfg(feature = "alloc")]
impl Numeric {
    /// Lossy string conversion for all numeric variants.
    ///
    /// This is intended for formatting and key generation where
    /// a human-readable representation is sufficient.
    #[inline]
    pub fn to_string_lossy(&self) -> alloc::string::String {
        match self {
            Numeric::Integer(i) => i.to_string(),
            Numeric::Float(f) => f.to_string(),
            Numeric::UInteger(u) => u.to_string(),
            Numeric::Byte(b) => b.to_string(),
            Numeric::Int32(i) => i.to_string(),
            Numeric::UInt32(u) => u.to_string(),
            Numeric::Int16(i) => i.to_string(),
            Numeric::UInt16(u) => u.to_string(),
            Numeric::Int8(i) => i.to_string(),
            Numeric::UInt8(u) => u.to_string(),
        }
    }
}

/// Embedded systems helper methods for Numeric
#[cfg(feature = "embedded")]
impl Numeric {
    /// Convert to i32, recommended for embedded systems
    ///
    /// This method provides safe conversion of all numeric types to i32,
    /// which is typically the most efficient integer type on 32-bit embedded platforms.
    ///
    /// Returns None if the value cannot fit in an i32.
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Numeric::Integer(v) => i32::try_from(*v).ok(),
            Numeric::Float(v) => {
                if v.is_finite() && *v >= i32::MIN as f64 && *v <= i32::MAX as f64 {
                    Some(*v as i32)
                } else {
                    None
                }
            }
            #[cfg(feature = "alloc")]
            Numeric::UInteger(v) => i32::try_from(*v).ok(),
            Numeric::Byte(v) => Some(*v as i32),
            Numeric::Int32(v) => Some(*v),
            Numeric::UInt32(v) => i32::try_from(*v).ok(),
            Numeric::Int16(v) => Some(*v as i32),
            Numeric::UInt16(v) => Some(*v as i32),
            Numeric::Int8(v) => Some(*v as i32),
            Numeric::UInt8(v) => Some(*v as i32),
        }
    }

    /// Convert to f32, recommended for embedded systems
    ///
    /// This method provides conversion of all numeric types to f32,
    /// which is typically the most efficient floating-point type on embedded platforms.
    ///
    /// Note: Conversion from 64-bit types may lose precision.
    pub fn to_f32(&self) -> f32 {
        match self {
            Numeric::Integer(v) => *v as f32,
            Numeric::Float(v) => *v as f32,
            Numeric::UInteger(v) => *v as f32,
            Numeric::Byte(v) => *v as f32,
            Numeric::Int32(v) => *v as f32,
            Numeric::UInt32(v) => *v as f32,
            Numeric::Int16(v) => *v as f32,
            Numeric::UInt16(v) => *v as f32,
            Numeric::Int8(v) => *v as f32,
            Numeric::UInt8(v) => *v as f32,
        }
    }

    /// Check if this numeric value fits in i32 range
    ///
    /// Returns true if the value can be safely converted to i32 without loss.
    pub fn fits_in_i32(&self) -> bool {
        self.to_i32().is_some()
    }

    /// Get the memory size of this numeric variant in bytes
    ///
    /// Useful for memory accounting in embedded systems.
    pub fn size_bytes(&self) -> usize {
        match self {
            Numeric::Integer(_) => 8,
            Numeric::Float(_) => 8,
            Numeric::UInteger(_) => 8,
            Numeric::Byte(_) => 1,
            Numeric::Int32(_) => 4,
            Numeric::UInt32(_) => 4,
            Numeric::Int16(_) => 2,
            Numeric::UInt16(_) => 2,
            Numeric::Int8(_) => 1,
            Numeric::UInt8(_) => 1,
        }
    }
}

impl From<i64> for Node {
    fn from(value: i64) -> Self {
        Node::Number(Numeric::Integer(value))
    }
}

#[cfg(feature = "alloc")]
impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Str(
            alloc::string::String::from(value),
            QuoteType::Unquoted,
            BlockStyle::None,
        )
    }
}

impl From<f64> for Node {
    fn from(value: f64) -> Self {
        Node::Number(Numeric::Float(value))
    }
}

impl From<u64> for Node {
    fn from(value: u64) -> Self {
        Node::Number(Numeric::UInteger(value))
    }
}

impl From<u8> for Node {
    fn from(value: u8) -> Self {
        Node::Number(Numeric::Byte(value))
    }
}

impl From<i32> for Node {
    fn from(value: i32) -> Self {
        Node::Number(Numeric::Int32(value))
    }
}

impl From<u32> for Node {
    fn from(value: u32) -> Self {
        Node::Number(Numeric::UInt32(value))
    }
}

impl From<i16> for Node {
    fn from(value: i16) -> Self {
        Node::Number(Numeric::Int16(value))
    }
}

impl From<u16> for Node {
    fn from(value: u16) -> Self {
        Node::Number(Numeric::UInt16(value))
    }
}

impl From<i8> for Node {
    fn from(value: i8) -> Self {
        Node::Number(Numeric::Int8(value))
    }
}

impl From<bool> for Node {
    fn from(value: bool) -> Self {
        Node::Boolean(value)
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for Node {
    fn from(value: alloc::string::String) -> Self {
        Node::Str(value, QuoteType::Unquoted, BlockStyle::None)
    }
}

/// Safe indexing and access methods for Node (panic-free)
///
/// These methods provide safe access to Node contents without panicking.
/// They return Option types to handle missing values or type mismatches gracefully.
/// These methods are recommended for production code and required for embedded systems.
#[cfg(feature = "alloc")]
impl Node {
    /// Returns true if the node is considered blank (None, empty array, empty string, comment, or recursively blank document/anchored node)
    #[inline]
    pub fn is_blank(&self) -> bool {
        match self {
            Node::None => true,
            Node::Array(items) => items.is_empty(),
            Node::Mapping(_pairs) => false,
            Node::Document(nodes) => nodes.iter().all(|n| n.is_blank()),
            Node::Str(s, _, _) => s.is_empty(),
            Node::Comment(_) => true,
            Node::Anchored(inner, _name) => (**inner).is_blank(),
            Node::Alias(_name) => false,
            _ => false,
        }
    }
    /// Safely get an array element by index without panicking
    ///
    /// Returns None if the index is out of bounds or if the node is not an array/set.
    /// This is the recommended method to avoid panics in production code.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// assert!(array.get(0).is_some());
    /// assert!(array.get(5).is_none());
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Node> {
        match self {
            Node::Array(arr) => arr.get(index),
            Node::Set(set) => set.get(index),
            _ => None,
        }
    }

    /// Safely get a mapping value by key without panicking
    ///
    /// Returns None if the key doesn't exist or if the node is not a mapping.
    /// This is the recommended method to avoid panics in production code.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// assert!(mapping.get_key("key").is_some());
    /// assert!(mapping.get_key("nonexistent").is_none());
    /// ```
    #[inline]
    pub fn get_key(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return Some(v);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Safely get a mutable array element by index without panicking
    ///
    /// Returns None if the index is out of bounds or if the node is not an array/set.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mut array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// if let Some(node) = array.get_mut(0) {
    ///     *node = Node::from(10);
    /// }
    /// ```
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Node> {
        match self {
            Node::Array(arr) => arr.get_mut(index),
            Node::Set(set) => set.get_mut(index),
            _ => None,
        }
    }

    /// Safely get a mutable mapping value by key without panicking
    ///
    /// Returns None if the key doesn't exist or if the node is not a mapping.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mut mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// if let Some(node) = mapping.get_key_mut("key") {
    ///     *node = Node::from("new_value");
    /// }
    /// ```
    #[inline]
    pub fn get_key_mut(&mut self, key: &str) -> Option<&mut Node> {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs.iter_mut() {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return Some(v);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Check if this node is an array or set
    ///
    /// Returns true for Node::Array and Node::Set variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![]);
    /// assert!(array.is_sequence());
    /// let mapping = Node::Mapping(vec![]);
    /// assert!(!mapping.is_sequence());
    /// ```
    #[inline]
    pub fn is_sequence(&self) -> bool {
        matches!(self, Node::Array(_) | Node::Set(_))
    }

    /// Check if this node is a mapping
    ///
    /// Returns true for Node::Mapping variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mapping = Node::Mapping(vec![]);
    /// assert!(mapping.is_mapping());
    /// let array = Node::Array(vec![]);
    /// assert!(!array.is_mapping());
    /// ```
    #[inline]
    pub fn is_mapping(&self) -> bool {
        matches!(self, Node::Mapping(_))
    }

    /// Get the length of an array, set, or mapping
    ///
    /// Returns None if the node is not a collection type.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// assert_eq!(array.len(), Some(2));
    /// let scalar = Node::from(42);
    /// assert_eq!(scalar.len(), None);
    /// ```
    #[inline]
    pub fn len(&self) -> Option<usize> {
        match self {
            Node::Array(arr) => Some(arr.len()),
            Node::Set(set) => Some(set.len()),
            Node::Mapping(pairs) => Some(pairs.len()),
            _ => None,
        }
    }

    /// Check if a collection is empty
    ///
    /// Returns true if the node is a collection and is empty, false otherwise.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![]);
    /// assert!(array.is_empty());
    /// let array = Node::Array(vec![Node::from(1)]);
    /// assert!(!array.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len().map_or(false, |l| l == 0)
    }

    /// Safely convert a numeric node to i32
    ///
    /// Returns None if the node is not numeric or if conversion fails.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let num = Node::from(42);
    /// assert_eq!(num.as_i32(), Some(42));
    /// let string = Node::from("text");
    /// assert_eq!(string.as_i32(), None);
    /// ```
    #[inline]
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            #[cfg(feature = "embedded")]
            Node::Number(num) => num.to_i32(),
            #[cfg(not(feature = "embedded"))]
            Node::Number(num) => match num {
                Numeric::Integer(v) => i32::try_from(*v).ok(),
                Numeric::Float(v) => {
                    if v.is_finite() && *v >= i32::MIN as f64 && *v <= i32::MAX as f64 {
                        Some(*v as i32)
                    } else {
                        None
                    }
                }
                Numeric::UInteger(v) => i32::try_from(*v).ok(),
                Numeric::Byte(v) => Some(*v as i32),
                Numeric::Int32(v) => Some(*v),
                Numeric::UInt32(v) => i32::try_from(*v).ok(),
                Numeric::Int16(v) => Some(*v as i32),
                Numeric::UInt16(v) => Some(*v as i32),
                Numeric::Int8(v) => Some(*v as i32),
                Numeric::UInt8(v) => Some(*v as i32),
            },
            _ => None,
        }
    }

    /// Safely convert a numeric node to f32
    ///
    /// Returns None if the node is not numeric.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let num = Node::from(3.14);
    /// assert_eq!(num.as_f32(), Some(3.14_f32));
    /// let string = Node::from("text");
    /// assert_eq!(string.as_f32(), None);
    /// ```
    #[inline]
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            #[cfg(feature = "embedded")]
            Node::Number(num) => Some(num.to_f32()),
            #[cfg(not(feature = "embedded"))]
            Node::Number(num) => Some(match num {
                Numeric::Integer(v) => *v as f32,
                Numeric::Float(v) => *v as f32,
                Numeric::UInteger(v) => *v as f32,
                Numeric::Byte(v) => *v as f32,
                Numeric::Int32(v) => *v as f32,
                Numeric::UInt32(v) => *v as f32,
                Numeric::Int16(v) => *v as f32,
                Numeric::UInt16(v) => *v as f32,
                Numeric::Int8(v) => *v as f32,
                Numeric::UInt8(v) => *v as f32,
            }),
            _ => None,
        }
    }

    /// Safely get a string value from a string node
    ///
    /// Returns None if the node is not a string.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let string = Node::from("hello");
    /// assert_eq!(string.as_str(), Some("hello"));
    /// let number = Node::from(42);
    /// assert_eq!(number.as_str(), None);
    /// ```
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s, _, _) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Safely get a boolean value
    ///
    /// Returns None if the node is not a boolean.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let bool_node = Node::from(true);
    /// assert_eq!(bool_node.as_bool(), Some(true));
    /// let number = Node::from(42);
    /// assert_eq!(number.as_bool(), None);
    /// ```
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if this node is a string
    ///
    /// Returns true for Node::Str variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let string = Node::from("hello");
    /// assert!(string.is_string());
    /// let number = Node::from(42);
    /// assert!(!number.is_string());
    /// ```
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Node::Str(_, _, _))
    }

    /// Check if this node is a number
    ///
    /// Returns true for Node::Number variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let number = Node::from(42);
    /// assert!(number.is_number());
    /// let string = Node::from("text");
    /// assert!(!string.is_number());
    /// ```
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Node::Number(_))
    }

    /// Check if this node is a boolean
    ///
    /// Returns true for Node::Boolean variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let bool_node = Node::from(true);
    /// assert!(bool_node.is_boolean());
    /// let number = Node::from(42);
    /// assert!(!number.is_boolean());
    /// ```
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Node::Boolean(_))
    }

    /// Alias for is_string() for consistency with as_str()
    ///
    /// Returns true for Node::Str variants.
    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, Node::Str(_, _, _))
    }

    /// Check if this node is an array
    ///
    /// Returns true for Node::Array variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![Node::from(1)]);
    /// assert!(array.is_array());
    /// let number = Node::from(42);
    /// assert!(!number.is_array());
    /// ```
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Node::Array(_))
    }

    /// Check if this node is a set
    ///
    /// Returns true for Node::Set variants.
    #[inline]
    pub fn is_set(&self) -> bool {
        matches!(self, Node::Set(_))
    }

    /// Check if this node is None (null)
    ///
    /// Returns true for Node::None variants.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let none = Node::None;
    /// assert!(none.is_none());
    /// let number = Node::from(42);
    /// assert!(!number.is_none());
    /// ```
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Node::None)
    }

    /// Try to get an array/set as a slice
    ///
    /// Returns None if the node is not an array or set.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// assert_eq!(array.as_slice().map(|s| s.len()), Some(2));
    /// let mapping = Node::Mapping(vec![]);
    /// assert!(mapping.as_slice().is_none());
    /// ```
    #[inline]
    pub fn as_slice(&self) -> Option<&[Node]> {
        match self {
            Node::Array(arr) => Some(arr.as_slice()),
            Node::Set(set) => Some(set.as_slice()),
            _ => None,
        }
    }

    /// Try to get a mapping as a slice of key-value pairs
    ///
    /// Returns None if the node is not a mapping.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// assert_eq!(mapping.as_mapping().map(|m| m.len()), Some(1));
    /// let array = Node::Array(vec![]);
    /// assert!(array.as_mapping().is_none());
    /// ```
    #[inline]
    pub fn as_mapping(&self) -> Option<&[(Node, Node)]> {
        match self {
            Node::Mapping(pairs) => Some(pairs.as_slice()),
            _ => None,
        }
    }

    /// Check if a mapping contains a specific key
    ///
    /// Returns false if the node is not a mapping.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// assert!(mapping.contains_key("key"));
    /// assert!(!mapping.contains_key("nonexistent"));
    /// ```
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.get_key(key).is_some()
    }

    /// Get all keys from a mapping as strings
    ///
    /// Returns an empty vector if the node is not a mapping or if keys are not strings.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key1"), Node::from("value1")),
    ///     (Node::from("key2"), Node::from("value2"))
    /// ]);
    /// let keys = mapping.keys();
    /// assert_eq!(keys.len(), 2);
    /// assert!(keys.contains(&"key1"));
    /// ```
    pub fn keys(&self) -> alloc::vec::Vec<&str> {
        match self {
            Node::Mapping(pairs) => {
                let mut keys = alloc::vec::Vec::with_capacity(pairs.len());
                for (k, _) in pairs {
                    if let Node::Str(s, _, _) = k {
                        keys.push(s.as_str());
                    }
                }
                keys
            }
            _ => alloc::vec::Vec::new(),
        }
    }

    // ==================== Tree Traversal Methods ====================

    /// Iterate over all immediate child nodes
    ///
    /// Returns an iterator over references to child nodes. Does not recurse.
    /// For recursive traversal, use `visit()` or `iter_all()`.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// let children: Vec<_> = array.children().collect();
    /// assert_eq!(children.len(), 2);
    /// ```
    pub fn children(&self) -> NodeChildIterator {
        NodeChildIterator::new(self)
    }

    /// Visit all nodes in the tree with a closure (depth-first, pre-order)
    ///
    /// The closure receives a reference to each node and its depth in the tree.
    /// Root node is at depth 0. Returns early if the closure returns false.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::Array(vec![Node::from(2), Node::from(3)])
    /// ]);
    ///
    /// let mut count = 0;
    /// doc.visit(|node, depth| {
    ///     count += 1;
    ///     true // continue traversal
    /// });
    /// assert_eq!(count, 5); // root array + 1 + nested array + 2 + 3
    /// ```
    pub fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(&Node, usize) -> bool,
    {
        self.visit_internal(&mut visitor, 0);
    }

    fn visit_internal<F>(&self, visitor: &mut F, depth: usize) -> bool
    where
        F: FnMut(&Node, usize) -> bool,
    {
        // Visit this node first (pre-order)
        if !visitor(self, depth) {
            return false;
        }

        // Then visit children
        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    if !item.visit_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    if !key.visit_internal(visitor, depth + 1) {
                        return false;
                    }
                    if !value.visit_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                return inner.visit_internal(visitor, depth + 1);
            }
            _ => {}
        }

        true
    }

    /// Visit all nodes in the tree with mutable access (depth-first, pre-order)
    ///
    /// The closure receives a mutable reference to each node and its depth.
    /// Allows modification of nodes during traversal.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// # use yaml_lib::Numeric;
    /// let mut doc = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// doc.visit_mut(|node, _depth| {
    ///     if let Node::Number(n) = node {
    ///         // Could modify number here
    ///         if let Numeric::Int32(val) = n {
    ///             *val *= 2; // Double the value
    ///         }
    ///     }
    ///     true
    /// });
    /// ```
    pub fn visit_mut<F>(&mut self, mut visitor: F)
    where
        F: FnMut(&mut Node, usize) -> bool,
    {
        self.visit_mut_internal(&mut visitor, 0);
    }

    fn visit_mut_internal<F>(&mut self, visitor: &mut F, depth: usize) -> bool
    where
        F: FnMut(&mut Node, usize) -> bool,
    {
        // Visit this node first (pre-order)
        if !visitor(self, depth) {
            return false;
        }

        // Then visit children
        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    if !item.visit_mut_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    if !key.visit_mut_internal(visitor, depth + 1) {
                        return false;
                    }
                    if !value.visit_mut_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                return inner.visit_mut_internal(visitor, depth + 1);
            }
            _ => {}
        }

        true
    }

    /// Count all nodes in the tree (including this node)
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::Array(vec![Node::from(2)])
    /// ]);
    /// assert_eq!(doc.count_nodes(), 4); // array + 1 + nested array + 2
    /// ```
    pub fn count_nodes(&self) -> usize {
        let mut count = 0;
        self.visit(|_, _| {
            count += 1;
            true
        });
        count
    }

    /// Find the maximum depth of the tree
    ///
    /// Returns 0 for leaf nodes, 1+ for nodes with children.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let leaf = Node::from(42);
    /// assert_eq!(leaf.max_depth(), 0);
    ///
    /// let nested = Node::Array(vec![
    ///     Node::Array(vec![Node::from(1)])
    /// ]);
    /// assert_eq!(nested.max_depth(), 2);
    /// ```
    pub fn max_depth(&self) -> usize {
        let mut max = 0;
        self.visit(|_, depth| {
            if depth > max {
                max = depth;
            }
            true
        });
        max
    }

    /// Collect all nodes matching a predicate
    ///
    /// Returns a vector of references to nodes that match the predicate.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::from("text"),
    ///     Node::from(2)
    /// ]);
    ///
    /// let numbers = doc.find_all(|node| node.is_number());
    /// assert_eq!(numbers.len(), 2);
    /// ```
    pub fn find_all<F>(&self, mut predicate: F) -> alloc::vec::Vec<&Node>
    where
        F: FnMut(&Node) -> bool,
    {
        let mut results = alloc::vec::Vec::new();
        self.find_all_internal(&mut predicate, &mut results);
        results
    }

    fn find_all_internal<'a, F>(
        &'a self,
        predicate: &mut F,
        results: &mut alloc::vec::Vec<&'a Node>,
    ) where
        F: FnMut(&Node) -> bool,
    {
        if predicate(self) {
            results.push(self);
        }

        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    item.find_all_internal(predicate, results);
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    key.find_all_internal(predicate, results);
                    value.find_all_internal(predicate, results);
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                inner.find_all_internal(predicate, results);
            }
            _ => {}
        }
    }

    /// Find the first node matching a predicate
    ///
    /// Returns None if no matching node is found.
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from("text"),
    ///     Node::from(42)
    /// ]);
    ///
    /// let first_num = doc.find_first(|node| node.is_number());
    /// assert!(first_num.is_some());
    /// ```
    pub fn find_first<F>(&self, mut predicate: F) -> Option<&Node>
    where
        F: FnMut(&Node) -> bool,
    {
        self.find_first_internal(&mut predicate)
    }

    fn find_first_internal<F>(&self, predicate: &mut F) -> Option<&Node>
    where
        F: FnMut(&Node) -> bool,
    {
        if predicate(self) {
            return Some(self);
        }

        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    if let Some(found) = item.find_first_internal(predicate) {
                        return Some(found);
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    if let Some(found) = key.find_first_internal(predicate) {
                        return Some(found);
                    }
                    if let Some(found) = value.find_first_internal(predicate) {
                        return Some(found);
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                return inner.find_first_internal(predicate);
            }
            _ => {}
        }

        None
    }
}

/// Iterator over immediate child nodes
pub struct NodeChildIterator<'a> {
    node: &'a Node,
    index: usize,
    mapping_phase: usize, // 0 = keys, 1 = values, 2 = done
}

impl<'a> NodeChildIterator<'a> {
    fn new(node: &'a Node) -> Self {
        Self {
            node,
            index: 0,
            mapping_phase: 0,
        }
    }
}

impl<'a> Iterator for NodeChildIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.node {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                let result = items.get(self.index);
                self.index += 1;
                result
            }
            Node::Mapping(pairs) => {
                if self.mapping_phase == 0 {
                    // Return keys first
                    if let Some((key, _)) = pairs.get(self.index) {
                        self.index += 1;
                        return Some(key);
                    } else {
                        // Done with keys, move to values
                        self.mapping_phase = 1;
                        self.index = 0;
                    }
                }
                if self.mapping_phase == 1 {
                    // Return values
                    if let Some((_, value)) = pairs.get(self.index) {
                        self.index += 1;
                        return Some(value);
                    } else {
                        self.mapping_phase = 2;
                    }
                }
                None
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                if self.index == 0 {
                    self.index = 1;
                    Some(inner.as_ref())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ==================== Fluent Builder API ====================

/// Builder for constructing Array nodes with a fluent API
///
/// Provides a chainable interface for building arrays without nested
/// function calls or manual vector construction.
///
/// # Example
/// ```
/// # use yaml_lib::Node;
/// let array = Node::array()
///     .push(1)
///     .push(2)
///     .push("three")
///     .build();
/// ```
#[cfg(feature = "alloc")]
pub struct ArrayBuilder {
    items: alloc::vec::Vec<Node>,
}

#[cfg(feature = "alloc")]
impl ArrayBuilder {
    /// Create a new empty array builder
    pub fn new() -> Self {
        Self {
            items: alloc::vec::Vec::new(),
        }
    }

    /// Add an item to the array
    pub fn push<T: Into<Node>>(mut self, value: T) -> Self {
        self.items.push(value.into());
        self
    }

    /// Add multiple items to the array
    pub fn extend<T: Into<Node>>(mut self, values: impl IntoIterator<Item = T>) -> Self {
        self.items.extend(values.into_iter().map(|v| v.into()));
        self
    }

    /// Add an item only if a condition is true
    pub fn push_if<T: Into<Node>>(mut self, condition: bool, value: T) -> Self {
        if condition {
            self.items.push(value.into());
        }
        self
    }

    /// Add an item if Some, skip if None
    pub fn push_opt<T: Into<Node>>(mut self, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.items.push(v.into());
        }
        self
    }

    /// Build the final Array node
    pub fn build(self) -> Node {
        Node::Array(self.items)
    }

    /// Get the current number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the builder is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(feature = "alloc")]
impl Default for ArrayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing Mapping nodes with a fluent API
///
/// Provides a chainable interface for building mappings without nested
/// function calls or manual vector construction.
///
/// # Example
/// ```
/// # use yaml_lib::Node;
/// let config = Node::mapping()
///     .insert("name", "MyApp")
///     .insert("version", "1.0")
///     .insert("port", 8080)
///     .build();
/// ```
#[cfg(feature = "alloc")]
pub struct MappingBuilder {
    pairs: alloc::vec::Vec<(Node, Node)>,
}

#[cfg(feature = "alloc")]
impl MappingBuilder {
    /// Create a new empty mapping builder
    pub fn new() -> Self {
        Self {
            pairs: alloc::vec::Vec::new(),
        }
    }

    /// Insert a key-value pair
    pub fn insert<K: Into<Node>, V: Into<Node>>(mut self, key: K, value: V) -> Self {
        self.pairs.push((key.into(), value.into()));
        self
    }

    /// Insert a pair only if a condition is true
    pub fn insert_if<K: Into<Node>, V: Into<Node>>(
        mut self,
        condition: bool,
        key: K,
        value: V,
    ) -> Self {
        if condition {
            self.pairs.push((key.into(), value.into()));
        }
        self
    }

    /// Insert a pair if value is Some, skip if None
    pub fn insert_opt<K: Into<Node>, V: Into<Node>>(mut self, key: K, value: Option<V>) -> Self {
        if let Some(v) = value {
            self.pairs.push((key.into(), v.into()));
        }
        self
    }

    /// Insert or update a key-value pair (replaces existing key)
    pub fn upsert<K: Into<Node>, V: Into<Node>>(mut self, key: K, value: V) -> Self {
        let key_node = key.into();
        let value_node = value.into();

        // Find and replace existing key, or insert new
        let mut found = false;
        for (k, v) in self.pairs.iter_mut() {
            if k == &key_node {
                *v = value_node.clone();
                found = true;
                break;
            }
        }

        if !found {
            self.pairs.push((key_node, value_node));
        }

        self
    }

    /// Build the final Mapping node
    pub fn build(self) -> Node {
        Node::Mapping(self.pairs)
    }

    /// Get the current number of pairs
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Check if the builder is empty
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.pairs.iter().any(|(k, _)| {
            if let Node::Str(s, _, _) = k {
                s == key
            } else {
                false
            }
        })
    }
}

#[cfg(feature = "alloc")]
impl Default for MappingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing Set nodes with a fluent API
///
/// Automatically ensures uniqueness of elements.
///
/// # Example
/// ```
/// # use yaml_lib::Node;
/// let set = Node::set()
///     .insert(1)
///     .insert(2)
///     .insert(1) // duplicate, will be ignored
///     .build();
/// ```
#[cfg(feature = "alloc")]
pub struct SetBuilder {
    items: alloc::vec::Vec<Node>,
}

#[cfg(feature = "alloc")]
impl SetBuilder {
    /// Create a new empty set builder
    pub fn new() -> Self {
        Self {
            items: alloc::vec::Vec::new(),
        }
    }

    /// Insert an item (duplicates are automatically ignored)
    pub fn insert<T: Into<Node>>(mut self, value: T) -> Self {
        let node = value.into();
        if !self.items.contains(&node) {
            self.items.push(node);
        }
        self
    }

    /// Insert multiple items (duplicates are automatically ignored)
    pub fn extend<T: Into<Node>>(mut self, values: impl IntoIterator<Item = T>) -> Self {
        for value in values {
            let node = value.into();
            if !self.items.contains(&node) {
                self.items.push(node);
            }
        }
        self
    }

    /// Build the final Set node
    pub fn build(self) -> Node {
        Node::Set(self.items)
    }

    /// Get the current number of unique items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the builder is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check if a value exists in the set
    pub fn contains(&self, value: &Node) -> bool {
        self.items.contains(value)
    }
}

#[cfg(feature = "alloc")]
impl Default for SetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods on Node for creating builders
#[cfg(feature = "alloc")]
impl Node {
    /// Create a new ArrayBuilder for fluent array construction
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let array = Node::array()
    ///     .push(1)
    ///     .push(2)
    ///     .push(3)
    ///     .build();
    /// ```
    pub fn array() -> ArrayBuilder {
        ArrayBuilder::new()
    }

    /// Create a new MappingBuilder for fluent mapping construction
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let config = Node::mapping()
    ///     .insert("host", "localhost")
    ///     .insert("port", 8080)
    ///     .build();
    /// ```
    pub fn mapping() -> MappingBuilder {
        MappingBuilder::new()
    }

    /// Create a new SetBuilder for fluent set construction
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::Node;
    /// let set = Node::set()
    ///     .insert(1)
    ///     .insert(2)
    ///     .insert(3)
    ///     .build();
    /// ```
    pub fn set() -> SetBuilder {
        SetBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_conversions() {
        assert_eq!(Numeric::from(42i64), Numeric::Integer(42));
        assert_eq!(Numeric::from(3.14f64), Numeric::Float(3.14));
        assert_eq!(Numeric::from(42u64), Numeric::UInteger(42));
        assert_eq!(Numeric::from(42u8), Numeric::Byte(42));
        assert_eq!(Numeric::from(42i32), Numeric::Int32(42));
        assert_eq!(Numeric::from(42u32), Numeric::UInt32(42));
        assert_eq!(Numeric::from(42i16), Numeric::Int16(42));
        assert_eq!(Numeric::from(42u16), Numeric::UInt16(42));
        assert_eq!(Numeric::from(42i8), Numeric::Int8(42));
    }

    #[test]
    fn test_node_numeric_conversions() {
        assert_eq!(Node::from(42i64), Node::Number(Numeric::Integer(42)));
        assert_eq!(Node::from(3.14f64), Node::Number(Numeric::Float(3.14)));
        assert_eq!(Node::from(42u64), Node::Number(Numeric::UInteger(42)));
        assert_eq!(Node::from(42u8), Node::Number(Numeric::Byte(42)));
        assert_eq!(Node::from(42i32), Node::Number(Numeric::Int32(42)));
        assert_eq!(Node::from(42u32), Node::Number(Numeric::UInt32(42)));
        assert_eq!(Node::from(42i16), Node::Number(Numeric::Int16(42)));
        assert_eq!(Node::from(42u16), Node::Number(Numeric::UInt16(42)));
        assert_eq!(Node::from(42i8), Node::Number(Numeric::Int8(42)));
    }

    #[test]
    fn test_node_string_conversions() {
        assert_eq!(
            Node::from("test"),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
        assert_eq!(
            Node::from("test".to_string()),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
    }

    #[test]
    fn test_node_bool_conversion() {
        assert_eq!(Node::from(true), Node::Boolean(true));
        assert_eq!(Node::from(false), Node::Boolean(false));
    }

    #[test]
    fn test_node_vec_conversion() {
        let vec = vec![1, 2, 3];
        let node = Node::from(vec);
        match node {
            Node::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(arr[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(arr[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Array node"),
        }
    }

    #[test]
    fn test_array_indexing() {
        let arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(arr[0], Node::Number(Numeric::Int32(1)));
        assert_eq!(arr[1], Node::Number(Numeric::Int32(2)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array/set node with integer")]
    fn test_invalid_array_indexing() {
        let node = Node::Boolean(true);
        let _value = &node[0];
    }

    #[test]
    fn test_mapping_indexing() {
        let obj = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(42),
        )]);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-mapping node with string")]
    fn test_invalid_mapping_indexing() {
        let node = Node::Boolean(true);
        let _value = &node["key"];
    }

    #[test]
    fn test_array_mut_indexing() {
        let mut arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        arr[0] = Node::from(42);
        assert_eq!(arr[0], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array/set node with integer")]
    fn test_invalid_array_mut_indexing() {
        let mut node = Node::Boolean(true);
        node[0] = Node::from(42);
    }

    #[test]
    fn test_mapping_mut_indexing() {
        let mut obj = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(42),
        )]);
        obj["key"] = Node::from(100);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(100)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-mapping node with string")]
    fn test_invalid_mapping_mut_indexing() {
        let mut node = Node::Boolean(true);
        node["key"] = Node::from(42);
    }

    #[test]
    #[should_panic(expected = "No such key exists")]
    fn test_mapping_mut_indexing_nonexistent_key() {
        let mut obj = Node::Mapping(Vec::new());
        obj["nonexistent"] = Node::from(42);
    }

    #[test]
    fn test_make_node() {
        assert_eq!(make_node(42), Node::Number(Numeric::Int32(42)));
        assert_eq!(
            make_node("test"),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
        assert_eq!(make_node(true), Node::Boolean(true));
    }
    #[test]
    fn test_make_node_vec() {
        let vec = vec![1, 2, 3];
        assert_eq!(
            make_node(vec),
            Node::Array(vec![
                Node::Number(Numeric::Int32(1)),
                Node::Number(Numeric::Int32(2)),
                Node::Number(Numeric::Int32(3))
            ])
        );
    }

    #[test]
    fn test_document_node() {
        let doc = Node::Documents(vec![Node::from(1), Node::from("test")]);
        match doc {
            Node::Documents(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert_eq!(nodes[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(
                    nodes[1],
                    Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
                );
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_comment_node() {
        let comment = Node::Comment("Test comment".to_string());
        match comment {
            Node::Comment(text) => assert_eq!(text, "Test comment"),
            _ => panic!("Expected Comment node"),
        }
    }

    #[test]
    fn test_none_node() {
        assert_eq!(Node::None, Node::None);
        let none = make_node(Node::None);
        assert_eq!(none, Node::None);
    }

    #[test]
    #[should_panic(expected = "No such key exists")]
    fn test_mapping_indexing_nonexistent_key() {
        let obj = Node::Mapping(Vec::new());
        let _ = &obj["nonexistent"];
    }

    #[test]
    fn test_node_from_vec_of_strings() {
        let vec = vec!["a", "b"];
        let node = Node::from(vec);
        assert_eq!(
            node,
            Node::Array(vec![
                Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("b".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ])
        );
    }

    #[test]
    fn test_nested_mapping_indexing_and_mutation() {
        let mut obj = Node::Mapping(vec![(
            Node::Str("outer".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![(
                Node::Str("inner".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::from(5),
            )]),
        )]);

        assert_eq!(obj["outer"]["inner"], Node::from(5));

        obj["outer"]["inner"] = Node::from(10);
        assert_eq!(obj["outer"]["inner"], Node::from(10));
    }

    #[test]
    fn test_set_node_creation() {
        let set = Node::Set(vec![Node::from(1), Node::from(2), Node::from(3)]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(items[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(items[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_set_indexing() {
        let set = Node::Set(vec![Node::from(1), Node::from(2)]);
        assert_eq!(set[0], Node::Number(Numeric::Int32(1)));
        assert_eq!(set[1], Node::Number(Numeric::Int32(2)));
    }

    #[test]
    fn test_set_mut_indexing() {
        let mut set = Node::Set(vec![Node::from(1), Node::from(2)]);
        set[0] = Node::from(42);
        assert_eq!(set[0], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array/set node with integer")]
    fn test_invalid_set_indexing() {
        let node = Node::Boolean(true);
        let _value = &node[0];
    }

    #[test]
    fn test_make_set_function() {
        let set = make_set(vec![1, 2, 3]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(items[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(items[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_make_set_with_duplicates() {
        let set = make_set(vec![1, 2, 2, 3, 1]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3); // Duplicates should be removed
                assert_eq!(items[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(items[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(items[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_make_set_with_strings() {
        let set = make_set(vec!["apple", "banana", "apple"]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 2); // Duplicate "apple" removed
                assert_eq!(
                    items[0],
                    Node::Str("apple".to_string(), QuoteType::Unquoted, BlockStyle::None)
                );
                assert_eq!(
                    items[1],
                    Node::Str("banana".to_string(), QuoteType::Unquoted, BlockStyle::None)
                );
            }
            _ => panic!("Expected Set node"),
        }
    }

    // Embedded feature tests
    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_to_i32() {
        let num_i32 = Numeric::Int32(42);
        assert_eq!(num_i32.to_i32(), Some(42));

        let num_i64 = Numeric::Integer(1000);
        assert_eq!(num_i64.to_i32(), Some(1000));

        let num_large = Numeric::Integer(i64::MAX);
        assert_eq!(num_large.to_i32(), None);

        let num_float = Numeric::Float(42.7);
        assert_eq!(num_float.to_i32(), Some(42));

        let num_byte = Numeric::Byte(255);
        assert_eq!(num_byte.to_i32(), Some(255));
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_to_f32() {
        let num_i32 = Numeric::Int32(42);
        assert_eq!(num_i32.to_f32(), 42.0f32);

        let num_float = Numeric::Float(3.14159);
        assert!((num_float.to_f32() - 3.14159f32).abs() < 0.0001);

        let num_i64 = Numeric::Integer(1000);
        assert_eq!(num_i64.to_f32(), 1000.0f32);
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_fits_in_i32() {
        assert!(Numeric::Int32(42).fits_in_i32());
        assert!(Numeric::Integer(1000).fits_in_i32());
        assert!(!Numeric::Integer(i64::MAX).fits_in_i32());
        assert!(Numeric::Float(100.5).fits_in_i32());
        assert!(Numeric::Byte(255).fits_in_i32());
        assert!(Numeric::Int16(1000).fits_in_i32());
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_size_bytes() {
        assert_eq!(Numeric::Integer(0).size_bytes(), 8);
        assert_eq!(Numeric::Float(0.0).size_bytes(), 8);
        assert_eq!(Numeric::UInteger(0).size_bytes(), 8);
        assert_eq!(Numeric::Int32(0).size_bytes(), 4);
        assert_eq!(Numeric::UInt32(0).size_bytes(), 4);
        assert_eq!(Numeric::Int16(0).size_bytes(), 2);
        assert_eq!(Numeric::UInt16(0).size_bytes(), 2);
        assert_eq!(Numeric::Byte(0).size_bytes(), 1);
        assert_eq!(Numeric::Int8(0).size_bytes(), 1);
    }

    #[test]
    fn test_node_get_safe() {
        let arr = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
        ]);

        assert_eq!(arr.get(0), Some(&Node::Number(Numeric::Int32(1))));
        assert_eq!(arr.get(1), Some(&Node::Number(Numeric::Int32(2))));
        assert_eq!(arr.get(2), Some(&Node::Number(Numeric::Int32(3))));
        assert_eq!(arr.get(3), None);

        let not_array = Node::Boolean(true);
        assert_eq!(not_array.get(0), None);
    }

    #[test]
    fn test_node_get_key_safe() {
        let mut pairs = alloc::vec::Vec::new();
        pairs.push((
            Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ));
        pairs.push((
            Node::Str("age".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Number(Numeric::Int32(42)),
        ));
        let mapping = Node::Mapping(pairs);

        assert_eq!(
            mapping.get_key("name"),
            Some(&Node::Str(
                "test".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None
            ))
        );
        assert_eq!(
            mapping.get_key("age"),
            Some(&Node::Number(Numeric::Int32(42)))
        );
        assert_eq!(mapping.get_key("nonexistent"), None);

        let not_mapping = Node::Boolean(true);
        assert_eq!(not_mapping.get_key("key"), None);
    }

    #[test]
    fn test_node_get_mut_safe() {
        let mut arr = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
        ]);

        if let Some(node) = arr.get_mut(0) {
            *node = Node::Number(Numeric::Int32(99));
        }

        assert_eq!(arr.get(0), Some(&Node::Number(Numeric::Int32(99))));
        assert_eq!(arr.get_mut(10), None);
    }

    #[test]
    fn test_node_is_sequence() {
        let arr = Node::Array(vec![Node::None]);
        let set = Node::Set(vec![Node::None]);
        let mapping = Node::Mapping(vec![]);
        let boolean = Node::Boolean(true);

        assert!(arr.is_sequence());
        assert!(set.is_sequence());
        assert!(!mapping.is_sequence());
        assert!(!boolean.is_sequence());
    }

    #[test]
    fn test_node_is_mapping() {
        let mapping = Node::Mapping(vec![]);
        let arr = Node::Array(vec![]);
        let boolean = Node::Boolean(true);

        assert!(mapping.is_mapping());
        assert!(!arr.is_mapping());
        assert!(!boolean.is_mapping());
    }

    #[test]
    fn test_node_len() {
        let arr = Node::Array(vec![Node::None, Node::None, Node::None]);
        let set = Node::Set(vec![Node::None, Node::None]);
        let mapping = Node::Mapping(vec![]);
        let boolean = Node::Boolean(true);

        assert_eq!(arr.len(), Some(3));
        assert_eq!(set.len(), Some(2));
        assert_eq!(mapping.len(), Some(0));
        assert_eq!(boolean.len(), None);
    }

    #[test]
    fn test_node_is_empty() {
        let arr_empty = Node::Array(vec![]);
        let arr_full = Node::Array(vec![Node::None]);
        let mapping_empty = Node::Mapping(vec![]);
        let boolean = Node::Boolean(true);

        assert!(arr_empty.is_empty());
        assert!(!arr_full.is_empty());
        assert!(mapping_empty.is_empty());
        assert!(!boolean.is_empty());
    }

    #[test]
    fn test_node_as_i32() {
        let num = Node::Number(Numeric::Int32(42));
        let large = Node::Number(Numeric::Integer(i64::MAX));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert_eq!(num.as_i32(), Some(42));
        assert_eq!(large.as_i32(), None);
        assert_eq!(string.as_i32(), None);
    }

    #[test]
    fn test_node_as_f32() {
        let num = Node::Number(Numeric::Float(3.14));
        let int = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!((num.as_f32().unwrap() - 3.14f32).abs() < 0.01);
        assert_eq!(int.as_f32(), Some(42.0f32));
        assert_eq!(string.as_f32(), None);
    }

    #[test]
    fn test_node_as_str() {
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let number = Node::Number(Numeric::Int32(42));

        assert_eq!(string.as_str(), Some("test"));
        assert_eq!(number.as_str(), None);
    }

    #[test]
    fn test_node_as_bool() {
        let bool_true = Node::Boolean(true);
        let bool_false = Node::Boolean(false);
        let number = Node::Number(Numeric::Int32(42));

        assert_eq!(bool_true.as_bool(), Some(true));
        assert_eq!(bool_false.as_bool(), Some(false));
        assert_eq!(number.as_bool(), None);
    }

    #[test]
    fn test_node_is_string() {
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let number = Node::Number(Numeric::Int32(42));
        let boolean = Node::Boolean(true);

        assert!(string.is_string());
        assert!(!number.is_string());
        assert!(!boolean.is_string());
    }

    #[test]
    fn test_node_is_number() {
        let number = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let boolean = Node::Boolean(true);

        assert!(number.is_number());
        assert!(!string.is_number());
        assert!(!boolean.is_number());
    }

    #[test]
    fn test_node_is_boolean() {
        let boolean = Node::Boolean(true);
        let number = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!(boolean.is_boolean());
        assert!(!number.is_boolean());
        assert!(!string.is_boolean());
    }

    #[test]
    fn test_node_is_none() {
        let none = Node::None;
        let number = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!(none.is_none());
        assert!(!number.is_none());
        assert!(!string.is_none());
    }

    #[test]
    fn test_node_as_slice() {
        let array = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
        ]);
        let set = Node::Set(vec![Node::Number(Numeric::Int32(1))]);
        let mapping = Node::Mapping(vec![]);

        assert_eq!(array.as_slice().map(|s| s.len()), Some(3));
        assert_eq!(set.as_slice().map(|s| s.len()), Some(1));
        assert!(mapping.as_slice().is_none());
    }

    #[test]
    fn test_node_as_mapping() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
        ]);
        let array = Node::Array(vec![]);

        assert_eq!(mapping.as_mapping().map(|m| m.len()), Some(2));
        assert!(array.as_mapping().is_none());
    }

    #[test]
    fn test_node_contains_key() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
        ]);
        let array = Node::Array(vec![]);

        assert!(mapping.contains_key("key1"));
        assert!(mapping.contains_key("key2"));
        assert!(!mapping.contains_key("key3"));
        assert!(!array.contains_key("key1"));
    }

    #[test]
    fn test_node_keys() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
            (Node::from("key3"), Node::from("value3")),
        ]);
        let array = Node::Array(vec![]);

        let keys = mapping.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1"));
        assert!(keys.contains(&"key2"));
        assert!(keys.contains(&"key3"));

        let empty_keys = array.keys();
        assert_eq!(empty_keys.len(), 0);
    }

    #[test]
    fn test_safe_access_prevents_panics() {
        // Test that safe methods don't panic on invalid access
        let array = Node::Array(vec![Node::from(1), Node::from(2)]);

        // Out of bounds access returns None instead of panicking
        assert!(array.get(100).is_none());

        // Wrong type access returns None instead of panicking
        let mapping = Node::Mapping(vec![]);
        assert!(mapping.get(0).is_none());

        // Nonexistent key returns None instead of panicking
        assert!(mapping.get_key("nonexistent").is_none());

        // Type check before access
        let scalar = Node::from(42);
        if scalar.is_sequence() {
            let _ = scalar.get(0); // Won't execute
        }
        assert!(!scalar.is_sequence());
    }

    #[test]
    fn test_safe_mutable_access() {
        let mut array = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        // Modify existing elements
        if let Some(node) = array.get_mut(1) {
            *node = Node::from(20);
        }
        assert_eq!(array.get(1), Some(&Node::Number(Numeric::Int32(20))));

        // Attempt to modify nonexistent element safely
        assert!(array.get_mut(100).is_none());

        // Mapping mutation
        let mut mapping = Node::Mapping(vec![(Node::from("key"), Node::from("value"))]);

        if let Some(node) = mapping.get_key_mut("key") {
            *node = Node::from("new_value");
        }
        assert_eq!(
            mapping.get_key("key"),
            Some(&Node::Str(
                "new_value".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None
            ))
        );
    }

    // ==================== Tree Traversal Tests ====================

    #[test]
    fn test_children_iterator_array() {
        let array = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
        let children: Vec<_> = array.children().collect();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], &Node::Number(Numeric::Int32(1)));
        assert_eq!(children[1], &Node::Number(Numeric::Int32(2)));
        assert_eq!(children[2], &Node::Number(Numeric::Int32(3)));
    }

    #[test]
    fn test_children_iterator_mapping() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from(1)),
            (Node::from("key2"), Node::from(2)),
        ]);
        let children: Vec<_> = mapping.children().collect();
        // Should return all keys first, then all values
        assert_eq!(children.len(), 4); // 2 keys + 2 values
    }

    #[test]
    fn test_children_iterator_leaf() {
        let leaf = Node::from(42);
        let children: Vec<_> = leaf.children().collect();
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_visit_simple_tree() {
        let doc = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        let mut visited = Vec::new();
        doc.visit(|node, depth| {
            visited.push((node.clone(), depth));
            true
        });

        assert_eq!(visited.len(), 4); // root + 3 children
        assert_eq!(visited[0].1, 0); // root at depth 0
        assert_eq!(visited[1].1, 1); // children at depth 1
    }

    #[test]
    fn test_visit_nested_tree() {
        let doc = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
        ]);

        let mut count = 0;
        let mut max_depth = 0;
        doc.visit(|_, depth| {
            count += 1;
            if depth > max_depth {
                max_depth = depth;
            }
            true
        });

        assert_eq!(count, 5); // root + 1 + nested array + 2 + 3
        assert_eq!(max_depth, 2);
    }

    #[test]
    fn test_visit_early_termination() {
        let doc = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        let mut count = 0;
        doc.visit(|_, _| {
            count += 1;
            count < 3 // stop after visiting 2 nodes
        });

        assert_eq!(count, 3); // visited 3 nodes before stopping
    }

    #[test]
    fn test_visit_mut_modification() {
        let mut doc = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        doc.visit_mut(|node, _| {
            if let Node::Number(Numeric::Int32(n)) = node {
                *n *= 2; // double all numbers
            }
            true
        });

        assert_eq!(doc[0], Node::Number(Numeric::Int32(2)));
        assert_eq!(doc[1], Node::Number(Numeric::Int32(4)));
        assert_eq!(doc[2], Node::Number(Numeric::Int32(6)));
    }

    #[test]
    fn test_count_nodes() {
        let leaf = Node::from(42);
        assert_eq!(leaf.count_nodes(), 1);

        let array = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(array.count_nodes(), 3); // array + 2 numbers

        let nested = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
        ]);
        assert_eq!(nested.count_nodes(), 5); // outer array + 1 + inner array + 2 + 3
    }

    #[test]
    fn test_max_depth() {
        let leaf = Node::from(42);
        assert_eq!(leaf.max_depth(), 0);

        let array = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(array.max_depth(), 1);

        let nested = Node::Array(vec![Node::Array(vec![Node::Array(vec![Node::from(1)])])]);
        assert_eq!(nested.max_depth(), 3);
    }

    #[test]
    fn test_find_all() {
        let doc = Node::Array(vec![
            Node::from(1),
            Node::from("text"),
            Node::from(2),
            Node::Array(vec![Node::from(3)]),
        ]);

        let numbers = doc.find_all(|node| node.is_number());
        assert_eq!(numbers.len(), 3); // 1, 2, 3

        let strings = doc.find_all(|node| node.is_str());
        assert_eq!(strings.len(), 1); // "text"

        let arrays = doc.find_all(|node| node.is_array());
        assert_eq!(arrays.len(), 2); // outer and inner array
    }

    #[test]
    fn test_find_first() {
        let doc = Node::Array(vec![
            Node::from("first"),
            Node::from(1),
            Node::from("second"),
        ]);

        let first_str = doc.find_first(|node| node.is_str());
        assert!(first_str.is_some());
        if let Some(Node::Str(s, _, _)) = first_str {
            assert_eq!(s, "first"); // should find "first", not "second"
        } else {
            panic!("Expected string node");
        }

        let first_bool = doc.find_first(|node| node.is_boolean());
        assert!(first_bool.is_none());
    }

    #[test]
    fn test_find_all_with_mapping() {
        let doc = Node::Mapping(vec![
            (Node::from("key1"), Node::from(1)),
            (Node::from("key2"), Node::from("value")),
        ]);

        let numbers = doc.find_all(|node| node.is_number());
        assert_eq!(numbers.len(), 1); // just the number value

        let strings = doc.find_all(|node| node.is_str());
        assert_eq!(strings.len(), 3); // 2 keys + 1 value
    }

    #[test]
    fn test_children_with_anchored() {
        let anchored = Node::Anchored(alloc::boxed::Box::new(Node::from(42)), "anchor".to_string());
        let children: Vec<_> = anchored.children().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], &Node::Number(Numeric::Int32(42)));
    }

    #[test]
    fn test_visit_with_tagged() {
        let tagged = Node::Tagged(
            alloc::boxed::Box::new(Node::Array(vec![Node::from(1), Node::from(2)])),
            "!custom".to_string(),
        );

        let mut count = 0;
        tagged.visit(|_, _| {
            count += 1;
            true
        });

        assert_eq!(count, 4); // tagged + array + 2 numbers
    }

    #[test]
    fn test_find_deeply_nested() {
        let doc = Node::Mapping(vec![(
            Node::from("level1"),
            Node::Mapping(vec![(
                Node::from("level2"),
                Node::Array(vec![
                    Node::from(1),
                    Node::from(42), // target
                    Node::from(3),
                ]),
            )]),
        )]);

        let target = doc.find_first(|node| {
            if let Node::Number(Numeric::Int32(n)) = node {
                *n == 42
            } else {
                false
            }
        });

        assert!(target.is_some());
    }

    #[test]
    fn test_visit_mapping_order() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from(1)),
            (Node::from("key2"), Node::from(2)),
        ]);

        let mut visited_order = Vec::new();
        mapping.visit(|node, _| {
            if let Node::Number(Numeric::Int32(n)) = node {
                visited_order.push(*n);
            }
            true
        });

        // Numbers should be visited in mapping value order
        assert_eq!(visited_order, vec![1, 2]);
    }

    // ==================== Fluent API Builder Tests ====================

    #[test]
    fn test_array_builder_basic() {
        let array = Node::array().push(1).push(2).push(3).build();

        assert!(array.is_array());
        assert_eq!(array.len(), Some(3));
        assert_eq!(array[0], Node::from(1));
        assert_eq!(array[1], Node::from(2));
        assert_eq!(array[2], Node::from(3));
    }

    #[test]
    fn test_array_builder_mixed_types() {
        let array = Node::array()
            .push(42)
            .push("text")
            .push(true)
            .push(3.14)
            .build();

        assert_eq!(array.len(), Some(4));
        assert!(array[0].is_number());
        assert!(array[1].is_string());
        assert!(array[2].is_boolean());
        assert!(array[3].is_number());
    }

    #[test]
    fn test_array_builder_extend() {
        let array = Node::array().push(1).extend(vec![2, 3, 4]).push(5).build();

        assert_eq!(array.len(), Some(5));
        assert_eq!(array[0], Node::from(1));
        assert_eq!(array[4], Node::from(5));
    }

    #[test]
    fn test_array_builder_conditional() {
        let include_optional = true;
        let array = Node::array()
            .push(1)
            .push_if(include_optional, 2)
            .push_if(false, 999) // should not be added
            .push(3)
            .build();

        assert_eq!(array.len(), Some(3));
        assert_eq!(array[1], Node::from(2));
        assert_eq!(array[2], Node::from(3));
    }

    #[test]
    fn test_array_builder_optional() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;

        let array = Node::array()
            .push(1)
            .push_opt(some_value)
            .push_opt(none_value)
            .push(2)
            .build();

        assert_eq!(array.len(), Some(3));
        assert_eq!(array[1], Node::from(42));
        assert_eq!(array[2], Node::from(2));
    }

    #[test]
    fn test_array_builder_nested() {
        let nested = Node::array()
            .push(1)
            .push(Node::array().push(2).push(3).build())
            .push(4)
            .build();

        assert_eq!(nested.len(), Some(3));
        assert!(nested[1].is_array());
        assert_eq!(nested[1].len(), Some(2));
    }

    #[test]
    fn test_array_builder_empty() {
        let array = Node::array().build();
        assert!(array.is_empty());
        assert_eq!(array.len(), Some(0));
    }

    #[test]
    fn test_mapping_builder_basic() {
        let mapping = Node::mapping()
            .insert("name", "Alice")
            .insert("age", 30)
            .insert("active", true)
            .build();

        assert!(mapping.is_mapping());
        assert_eq!(mapping.len(), Some(3));
        assert_eq!(mapping["name"], Node::from("Alice"));
        assert_eq!(mapping["age"], Node::from(30));
        assert_eq!(mapping["active"], Node::from(true));
    }

    #[test]
    fn test_mapping_builder_nested() {
        let config = Node::mapping()
            .insert(
                "database",
                Node::mapping()
                    .insert("host", "localhost")
                    .insert("port", 5432)
                    .build(),
            )
            .insert("debug", false)
            .build();

        assert_eq!(config.len(), Some(2));
        assert!(config["database"].is_mapping());
        assert_eq!(config["database"]["host"], Node::from("localhost"));
    }

    #[test]
    fn test_mapping_builder_conditional() {
        let include_debug = true;
        let mapping = Node::mapping()
            .insert("name", "app")
            .insert_if(include_debug, "debug", true)
            .insert_if(false, "should_not_exist", 999)
            .build();

        assert_eq!(mapping.len(), Some(2));
        assert!(mapping.contains_key("debug"));
        assert!(!mapping.contains_key("should_not_exist"));
    }

    #[test]
    fn test_mapping_builder_optional() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<&str> = None;

        let mapping = Node::mapping()
            .insert("required", "value")
            .insert_opt("optional1", some_value)
            .insert_opt("optional2", none_value)
            .build();

        assert_eq!(mapping.len(), Some(2));
        assert!(mapping.contains_key("optional1"));
        assert!(!mapping.contains_key("optional2"));
    }

    #[test]
    fn test_mapping_builder_upsert() {
        let mapping = Node::mapping()
            .insert("key", "original")
            .insert("other", "value")
            .upsert("key", "updated") // should replace
            .upsert("new", "added") // should insert
            .build();

        assert_eq!(mapping.len(), Some(3));
        assert_eq!(mapping["key"], Node::from("updated"));
        assert_eq!(mapping["new"], Node::from("added"));
    }

    #[test]
    fn test_mapping_builder_complex() {
        let config = Node::mapping()
            .insert("name", "MyApp")
            .insert("version", "1.0.0")
            .insert(
                "servers",
                Node::array().push("web1").push("web2").push("web3").build(),
            )
            .insert(
                "database",
                Node::mapping()
                    .insert("host", "localhost")
                    .insert("port", 5432)
                    .insert("ssl", true)
                    .build(),
            )
            .build();

        assert_eq!(config.len(), Some(4));
        assert!(config["servers"].is_array());
        assert_eq!(config["servers"].len(), Some(3));
        assert!(config["database"].is_mapping());
        assert_eq!(config["database"]["port"], Node::from(5432));
    }

    #[test]
    fn test_set_builder_basic() {
        let set = Node::set().insert(1).insert(2).insert(3).build();

        assert!(set.is_set());
        assert_eq!(set.len(), Some(3));
    }

    #[test]
    fn test_set_builder_duplicates() {
        let set = Node::set()
            .insert(1)
            .insert(2)
            .insert(1) // duplicate
            .insert(3)
            .insert(2) // duplicate
            .build();

        assert_eq!(set.len(), Some(3)); // should only have 3 unique items
    }

    #[test]
    fn test_set_builder_extend() {
        let set = Node::set()
            .insert(1)
            .extend(vec![2, 3, 2, 4]) // includes duplicate 2
            .insert(5)
            .build();

        assert_eq!(set.len(), Some(5)); // 1, 2, 3, 4, 5
    }

    #[test]
    fn test_set_builder_mixed_types() {
        let set = Node::set().insert(1).insert("text").insert(true).build();

        assert_eq!(set.len(), Some(3));
    }

    #[test]
    fn test_builder_chaining_realistic_config() {
        // Realistic configuration example
        let config = Node::mapping()
            .insert(
                "application",
                Node::mapping()
                    .insert("name", "WebAPI")
                    .insert("version", "2.1.0")
                    .insert("environment", "production")
                    .build(),
            )
            .insert(
                "server",
                Node::mapping()
                    .insert("host", "0.0.0.0")
                    .insert("port", 8080)
                    .insert("workers", 4)
                    .build(),
            )
            .insert(
                "features",
                Node::array()
                    .push("auth")
                    .push("logging")
                    .push("metrics")
                    .build(),
            )
            .insert(
                "allowed_origins",
                Node::set()
                    .insert("https://example.com")
                    .insert("https://api.example.com")
                    .build(),
            )
            .build();

        // Verify structure
        assert_eq!(config.len(), Some(4));
        assert_eq!(config["application"]["name"], Node::from("WebAPI"));
        assert_eq!(config["server"]["port"], Node::from(8080));
        assert_eq!(config["features"].len(), Some(3));
        assert!(config["allowed_origins"].is_set());
    }

    #[test]
    fn test_builder_readability_comparison() {
        // Old way (verbose)
        let _old_way = Node::Mapping(vec![
            (
                Node::from("database"),
                Node::Mapping(vec![
                    (Node::from("host"), Node::from("localhost")),
                    (Node::from("port"), Node::from(5432)),
                ]),
            ),
            (
                Node::from("servers"),
                Node::Array(vec![Node::from("web1"), Node::from("web2")]),
            ),
        ]);

        // New way (fluent)
        let _new_way = Node::mapping()
            .insert(
                "database",
                Node::mapping()
                    .insert("host", "localhost")
                    .insert("port", 5432)
                    .build(),
            )
            .insert("servers", Node::array().push("web1").push("web2").build())
            .build();

        // Both should produce equivalent structures
        assert_eq!(_old_way, _new_way);
    }

    #[test]
    fn test_array_builder_len() {
        let builder = Node::array().push(1).push(2);

        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());

        let empty = Node::array();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_mapping_builder_contains_key() {
        let builder = Node::mapping()
            .insert("key1", "value1")
            .insert("key2", "value2");

        assert!(builder.contains_key("key1"));
        assert!(builder.contains_key("key2"));
        assert!(!builder.contains_key("key3"));
    }

    #[test]
    fn test_set_builder_contains() {
        let node1 = Node::from(1);
        let node2 = Node::from(2);
        let node3 = Node::from(3);

        let builder = Node::set().insert(1).insert(2);

        assert!(builder.contains(&node1));
        assert!(builder.contains(&node2));
        assert!(!builder.contains(&node3));
    }
    // --- New and relevant unit tests ---
    #[test]
    fn test_node_string_convert_trait() {
        let s = Node::from("hello");
        assert_eq!(s.to_string_lossy(), "hello");
        assert_eq!(s.as_str(), Some("hello"));
        assert_eq!(s.clone_as_string(), Some(Node::from("hello")));

        let n = Node::from(42);
        assert_eq!(n.to_string_lossy(), "42");
        assert_eq!(n.as_str(), None);
        assert_eq!(n.clone_as_string(), None);
    }

    #[test]
    fn test_numeric_variants_to_string_lossy() {
        let i = Node::from(-5i64);
        let f = Node::from(3.14f64);
        let u = Node::from(7u64);
        let b = Node::from(255u8);
        let i32v = Node::from(-123i32);
        assert_eq!(i.to_string_lossy(), "-5");
        assert_eq!(f.to_string_lossy(), "3.14");
        assert_eq!(u.to_string_lossy(), "7");
        assert_eq!(b.to_string_lossy(), "255");
        assert_eq!(i32v.to_string_lossy(), "-123");
    }

    #[test]
    fn test_node_equality_and_clone() {
        let n1 = Node::from("abc");
        let n2 = n1.clone();
        assert_eq!(n1, n2);
        let n3 = Node::from(123);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_node_array_and_mapping_index() {
        let arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(arr[0], Node::from(1));
        assert_eq!(arr[1], Node::from(2));

        let map = Node::mapping().insert("foo", 42).build();
        assert_eq!(map["foo"], Node::from(42));
    }

    #[test]
    fn test_node_set_insert_and_contains() {
        let set = Node::set().insert("a").insert("b");
        assert!(set.contains(&Node::from("a")));
        assert!(set.contains(&Node::from("b")));
        assert!(!set.contains(&Node::from("c")));
    }

    #[test]
    fn test_node_none_and_boolean() {
        let n = Node::None;
        assert_eq!(n.to_string_lossy(), "null");
        let t = Node::from(true);
        let f = Node::from(false);
        assert_eq!(t.to_string_lossy(), "true");
        assert_eq!(f.to_string_lossy(), "false");
    }
}
