//! Module: nodes/node.rs

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
}

/// Represents how a string was quoted in the source YAML
#[derive(Clone, Debug, PartialEq)]
/// QuoteType
pub enum QuoteType {
    Unquoted,
    Single,
    Double,
}

/// Represents whether a string originated from a block scalar and its style
#[derive(Clone, Debug, PartialEq)]
/// BlockStyle
pub enum BlockStyle {
    None,
    Literal,
    Folded,
}

/// A node in the YAML data structure that can represent different types of values.
#[derive(Clone, Debug, PartialEq)]
/// Node
#[cfg(feature = "alloc")]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg(not(feature = "alloc"))]
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
            Numeric::UInteger(v) => i32::try_from(*v).ok(),
            Numeric::Byte(v) => Some(*v as i32),
            Numeric::Int32(v) => Some(*v),
            Numeric::UInt32(v) => i32::try_from(*v).ok(),
            Numeric::Int16(v) => Some(*v as i32),
            Numeric::UInt16(v) => Some(*v as i32),
            Numeric::Int8(v) => Some(*v as i32),
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

/// Safe indexing and access methods for Node (embedded-safe, panic-free)
#[cfg(feature = "embedded")]
impl Node {
    /// Safely get an array element by index without panicking
    ///
    /// Returns None if the index is out of bounds or if the node is not an array/set.
    /// This is the recommended method for embedded systems to avoid panics.
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
    /// This is the recommended method for embedded systems to avoid panics.
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
    pub fn is_sequence(&self) -> bool {
        matches!(self, Node::Array(_) | Node::Set(_))
    }

    /// Check if this node is a mapping
    pub fn is_mapping(&self) -> bool {
        matches!(self, Node::Mapping(_))
    }

    /// Get the length of an array, set, or mapping
    ///
    /// Returns None if the node is not a collection type.
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
    pub fn is_empty(&self) -> bool {
        self.len().map_or(false, |l| l == 0)
    }

    /// Safely convert a numeric node to i32
    ///
    /// Returns None if the node is not numeric or if conversion fails.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Node::Number(num) => num.to_i32(),
            _ => None,
        }
    }

    /// Safely convert a numeric node to f32
    ///
    /// Returns None if the node is not numeric.
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Node::Number(num) => Some(num.to_f32()),
            _ => None,
        }
    }

    /// Safely get a string value from a string node
    ///
    /// Returns None if the node is not a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s, _, _) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Safely get a boolean value
    ///
    /// Returns None if the node is not a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Helper function to create a Set node from a vector, ensuring uniqueness
#[cfg(feature = "alloc")]
pub fn make_set<T>(values: alloc::vec::Vec<T>) -> Node
where
    T: Into<Node> + Clone,
{
    let mut unique_nodes = alloc::vec::Vec::new();

    for value in values {
        let node = value.into();
        if !unique_nodes.contains(&node) {
            unique_nodes.push(node);
        }
    }

    Node::Set(unique_nodes)
}

/// Helper functions to create a Node from any value that can be converted into a Node
pub fn make_node<T>(value: T) -> Node
where
    T: Into<Node>,
{
    value.into()
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
    #[cfg(feature = "embedded")]
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
    #[cfg(feature = "embedded")]
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
            Some(&Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None))
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
    #[cfg(feature = "embedded")]
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
    #[cfg(feature = "embedded")]
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
    #[cfg(feature = "embedded")]
    fn test_node_is_mapping() {
        let mapping = Node::Mapping(vec![]);
        let arr = Node::Array(vec![]);
        let boolean = Node::Boolean(true);

        assert!(mapping.is_mapping());
        assert!(!arr.is_mapping());
        assert!(!boolean.is_mapping());
    }

    #[test]
    #[cfg(feature = "embedded")]
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
    #[cfg(feature = "embedded")]
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
    #[cfg(feature = "embedded")]
    fn test_node_as_i32() {
        let num = Node::Number(Numeric::Int32(42));
        let large = Node::Number(Numeric::Integer(i64::MAX));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert_eq!(num.as_i32(), Some(42));
        assert_eq!(large.as_i32(), None);
        assert_eq!(string.as_i32(), None);
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_node_as_f32() {
        let num = Node::Number(Numeric::Float(3.14));
        let int = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!((num.as_f32().unwrap() - 3.14f32).abs() < 0.01);
        assert_eq!(int.as_f32(), Some(42.0f32));
        assert_eq!(string.as_f32(), None);
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_node_as_str() {
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let number = Node::Number(Numeric::Int32(42));

        assert_eq!(string.as_str(), Some("test"));
        assert_eq!(number.as_str(), None);
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_node_as_bool() {
        let bool_true = Node::Boolean(true);
        let bool_false = Node::Boolean(false);
        let number = Node::Number(Numeric::Int32(42));

        assert_eq!(bool_true.as_bool(), Some(true));
        assert_eq!(bool_false.as_bool(), Some(false));
        assert_eq!(number.as_bool(), None);
    }
}
