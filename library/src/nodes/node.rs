use std::collections::HashMap;
use std::ops::{Index, IndexMut};

/// Represents different numeric types that can be stored in a YAML node
#[derive(Clone, Debug, PartialEq)]
pub enum Numeric {
    Integer(i64),    // 64-bit signed integer
    Float(f64),      // 64-bit floating point
    UInteger(u64),   // 64-bit unsigned integer
    Byte(u8),        // 8-bit unsigned integer
    Int32(i32),      // 32-bit signed integer
    UInt32(u32),     // 32-bit unsigned integer
    Int16(i16),      // 16-bit signed integer
    UInt16(u16),     // 16-bit unsigned integer
    Int8(i8),        // 8-bit signed integer
}

/// A node in the YAML data structure that can represent different types of values.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// Represents a boolean value (true/false)
    /// Used for YAML boolean values like true/false, yes/no, on/off
    Boolean(bool),
    /// Represents a numeric value (various integer and float types)
    /// Stores numbers using the most appropriate numeric type from the Numeric enum
    Number(Numeric),
    /// Represents a string value 
    /// Used for text content in YAML including multi-line strings
    Str(String),
    /// Represents an array of other nodes
    /// Used for YAML sequences/lists where order matters
    Array(Vec<Node>),
    /// Represents a dictionary/map of string keys to node values
    /// Used for YAML mappings where keys map to values
    Dictionary(HashMap<String, Node>),
    /// Represents a comment
    /// Stores documentation and descriptive text that doesn't affect the data structure
    Comment(String),
    /// Represents a document node
    /// Contains a sequence of top-level nodes making up a YAML document
    Document(Vec<Node>),
    /// Represents a null value or uninitialized node
    /// Used for explicit null values in YAML or missing/undefined values
    None,
}

/// Implements array-style indexing for Node using integer indices
impl Index<usize> for Node {
    type Output = Node;

    /// Allows accessing array elements using array[index] syntax
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node::Array(arr) => &arr[index],
            _ => panic!("Cannot index non-array node with integer"),
        }
    }
}

/// Implements dictionary-style indexing for Node using string keys
impl Index<&str> for Node {
    type Output = Node;

    /// Allows accessing dictionary properties using dictionary["key"] syntax
    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Node::Dictionary(map) => &map[key],
            _ => panic!("Cannot index non-dictionary node with string"),
        }
    }
}

/// Implements mutable array-style indexing for Node
impl IndexMut<usize> for Node {
    /// Allows modifying array elements using array[index] = value syntax
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Node::Array(arr) => &mut arr[index],
            _ => panic!("Cannot index non-array node with integer"),
        }
    }
}

/// Implements mutable dictionary-style indexing for Node
impl IndexMut<&str> for Node {
    /// Allows modifying dictionary properties using dictionary["key"] = value syntax
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        match self {
            Node::Dictionary(map) => map.get_mut(key).expect("No such key exists"),
            _ => panic!("Cannot index non-dictionary node with string"),
        }
    }
}

/// Converts a vector of values into an array node
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

impl From<i64> for Node {
    fn from(value: i64) -> Self {
        Node::Number(Numeric::Integer(value))
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Str(String::from(value))
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

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::Str(value)
    }
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
        assert_eq!(Node::from("test"), Node::Str("test".to_string()));
        assert_eq!(Node::from("test".to_string()), Node::Str("test".to_string()));
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
    #[should_panic(expected = "Cannot index non-array node with integer")]
    fn test_invalid_array_indexing() {
        let node = Node::Boolean(true);
        let _value = &node[0];
    }

    #[test]
    fn test_dictionary_indexing() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::from(42));
        let obj = Node::Dictionary(map);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-dictionary node with string")]
    fn test_invalid_dictionary_indexing() {
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
    #[should_panic(expected = "Cannot index non-array node with integer")]
    fn test_invalid_array_mut_indexing() {
        let mut node = Node::Boolean(true);
        node[0] = Node::from(42);
    }

    #[test]
    fn test_dictionary_mut_indexing() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::from(42));
        let mut obj = Node::Dictionary(map);
        obj["key"] = Node::from(100);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(100)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-dictionary node with string")]
    fn test_invalid_dictionary_mut_indexing() {
        let mut node = Node::Boolean(true);
        node["key"] = Node::from(42);
    }

    #[test]
    #[should_panic(expected = "No such key exists")]
    fn test_dictionary_mut_indexing_nonexistent_key() {
        let mut obj = Node::Dictionary(HashMap::new());
        obj["nonexistent"] = Node::from(42);
    }

    #[test]
    fn test_make_node() {
        assert_eq!(make_node(42), Node::Number(Numeric::Int32(42)));
        assert_eq!(make_node("test"), Node::Str("test".to_string()));
        assert_eq!(make_node(true), Node::Boolean(true));
    }
    #[test]
    fn test_make_node_vec() {
        let vec = vec![1, 2, 3];
        assert_eq!(make_node(vec), Node::Array(vec![Node::Number(Numeric::Int32(1)), Node::Number(Numeric::Int32(2)), Node::Number(Numeric::Int32(3))]));
    }

    #[test]
    fn test_document_node() {
        let doc = Node::Document(vec![Node::from(1), Node::from("test")]);
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert_eq!(nodes[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(nodes[1], Node::Str("test".to_string()));
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
}
