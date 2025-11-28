//! Property-based testing utilities
//!
//! Provides tools for defining and checking properties that should always hold.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::nodes::node::Node;

/// Property test result
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyResult {
    /// Property holds
    Pass,
    /// Property violated
    Fail(String),
    /// Property could not be tested
    Skip(String),
}

/// Property test function
pub type PropertyFn = fn(&Node) -> PropertyResult;

/// Property test
pub struct Property {
    name: String,
    test_fn: PropertyFn,
}

impl Property {
    /// Create new property test
    pub fn new(name: impl Into<String>, test_fn: PropertyFn) -> Self {
        Self {
            name: name.into(),
            test_fn,
        }
    }

    /// Check if property holds for given node
    pub fn check(&self, node: &Node) -> PropertyResult {
        (self.test_fn)(node)
    }

    /// Get property name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Common YAML properties that should always hold
pub mod properties {
    use super::*;
    use crate::io::sources::buffer::Buffer as BufferSource;
    use crate::parser::document::parse;
    use crate::stringify::default::stringify;

    /// Property: Parse never panics
    pub fn parse_never_panics(node: &Node) -> PropertyResult {
        let mut buffer = crate::io::destinations::buffer::Buffer::new();
        let yaml = match stringify(node, &mut buffer) {
            Ok(_) => buffer.to_string(),
            Err(_) => return PropertyResult::Skip("Could not stringify".to_string()),
        };

        let mut source = BufferSource::new(yaml.as_bytes());
        match parse(&mut source) {
            Ok(_) | Err(_) => PropertyResult::Pass,
        }
    }

    /// Property: Stringify never panics
    pub fn stringify_never_panics(node: &Node) -> PropertyResult {
        let mut buffer = crate::io::destinations::buffer::Buffer::new();
        match stringify(node, &mut buffer) {
            Ok(_) | Err(_) => PropertyResult::Pass,
        }
    }

    /// Property: Round-trip preserves structure (parse -> stringify -> parse)
    pub fn roundtrip_preserves_structure(node: &Node) -> PropertyResult {
        // First stringify
        let mut buffer1 = crate::io::destinations::buffer::Buffer::new();
        let yaml1 = match stringify(node, &mut buffer1) {
            Ok(_) => buffer1.to_string(),
            Err(_) => return PropertyResult::Skip("Could not stringify".to_string()),
        };

        // Parse
        let mut source1 = BufferSource::new(yaml1.as_bytes());
        let node1 = match parse(&mut source1) {
            Ok(n) => n,
            Err(e) => return PropertyResult::Fail(format!("Parse failed: {}", e)),
        };

        // Stringify again
        let mut buffer2 = crate::io::destinations::buffer::Buffer::new();
        let yaml2 = match stringify(&node1, &mut buffer2) {
            Ok(_) => buffer2.to_string(),
            Err(_) => return PropertyResult::Fail("Second stringify failed".to_string()),
        };

        // Parse again
        let mut source2 = BufferSource::new(yaml2.as_bytes());
        let node2 = match parse(&mut source2) {
            Ok(n) => n,
            Err(e) => return PropertyResult::Fail(format!("Second parse failed: {}", e)),
        };

        // Compare debug representations (not perfect but good enough)
        let repr1 = format!("{:?}", node1);
        let repr2 = format!("{:?}", node2);

        if repr1 == repr2 {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail(format!(
                "Round-trip changed structure:\nFirst:  {:?}\nSecond: {:?}\nYAML1: {}\nYAML2: {}",
                node1, node2, yaml1, yaml2
            ))
        }
    }

    /// Property: Node depth is bounded
    pub fn depth_is_bounded(node: &Node) -> PropertyResult {
        fn check_depth(node: &Node, current: usize, max: usize) -> bool {
            if current > max {
                return false;
            }

            match node {
                Node::Array(items) => items.iter().all(|n| check_depth(n, current + 1, max)),
                Node::Mapping(pairs) => pairs.iter().all(|(k, v)| {
                    check_depth(k, current + 1, max) && check_depth(v, current + 1, max)
                }),
                Node::Set(items) => items.iter().all(|n| check_depth(n, current + 1, max)),
                Node::Document(items) => items.iter().all(|n| check_depth(n, current + 1, max)),
                Node::Documents(docs) => docs.iter().all(|n| check_depth(n, current + 1, max)),
                Node::Tagged(inner, _) => check_depth(inner, current + 1, max),
                Node::Anchored(inner, _) => check_depth(inner, current + 1, max),
                _ => true,
            }
        }

        if check_depth(node, 0, 1000) {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail("Depth exceeds 1000".to_string())
        }
    }

    /// Property: No circular references
    pub fn no_circular_references(node: &Node) -> PropertyResult {
        use alloc::vec::Vec;
        use core::ptr;

        fn check_circular(node: &Node, visited: &mut Vec<*const Node>) -> bool {
            let node_ptr = node as *const Node;

            if visited.iter().any(|&p| ptr::eq(p, node_ptr)) {
                return false;
            }

            visited.push(node_ptr);

            let result = match node {
                Node::Array(items) => items.iter().all(|n| check_circular(n, visited)),
                Node::Mapping(pairs) => pairs
                    .iter()
                    .all(|(k, v)| check_circular(k, visited) && check_circular(v, visited)),
                Node::Set(items) => items.iter().all(|n| check_circular(n, visited)),
                Node::Document(items) => items.iter().all(|n| check_circular(n, visited)),
                Node::Documents(docs) => docs.iter().all(|n| check_circular(n, visited)),
                Node::Tagged(inner, _) => check_circular(inner, visited),
                Node::Anchored(inner, _) => check_circular(inner, visited),
                _ => true,
            };

            visited.pop();
            result
        }

        let mut visited = Vec::new();
        if check_circular(node, &mut visited) {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail("Circular reference detected".to_string())
        }
    }

    /// Property: Collections are finite
    pub fn collections_are_finite(node: &Node) -> PropertyResult {
        fn check_finite(node: &Node, max_size: usize) -> bool {
            match node {
                Node::Array(items) => {
                    items.len() <= max_size && items.iter().all(|n| check_finite(n, max_size))
                }
                Node::Mapping(pairs) => {
                    pairs.len() <= max_size
                        && pairs
                            .iter()
                            .all(|(k, v)| check_finite(k, max_size) && check_finite(v, max_size))
                }
                Node::Set(items) => {
                    items.len() <= max_size && items.iter().all(|n| check_finite(n, max_size))
                }
                Node::Document(items) => {
                    items.len() <= max_size && items.iter().all(|n| check_finite(n, max_size))
                }
                Node::Documents(docs) => {
                    docs.len() <= max_size && docs.iter().all(|n| check_finite(n, max_size))
                }
                Node::Tagged(inner, _) => check_finite(inner, max_size),
                Node::Anchored(inner, _) => check_finite(inner, max_size),
                _ => true,
            }
        }

        if check_finite(node, 10000) {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail("Collection exceeds size limit".to_string())
        }
    }
}

/// Property test suite
pub struct PropertySuite {
    properties: Vec<Property>,
}

impl PropertySuite {
    /// Create new property suite
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }

    /// Create suite with common properties
    pub fn common() -> Self {
        let mut suite = Self::new();
        suite.add(Property::new(
            "parse_never_panics",
            properties::parse_never_panics,
        ));
        suite.add(Property::new(
            "stringify_never_panics",
            properties::stringify_never_panics,
        ));
        suite.add(Property::new(
            "depth_is_bounded",
            properties::depth_is_bounded,
        ));
        suite.add(Property::new(
            "no_circular_references",
            properties::no_circular_references,
        ));
        suite.add(Property::new(
            "collections_are_finite",
            properties::collections_are_finite,
        ));
        suite
    }

    /// Add property to suite
    pub fn add(&mut self, property: Property) {
        self.properties.push(property);
    }

    /// Check all properties for a node
    pub fn check_all(&self, node: &Node) -> Vec<(String, PropertyResult)> {
        self.properties
            .iter()
            .map(|prop| (prop.name().to_string(), prop.check(node)))
            .collect()
    }

    /// Check if all properties pass
    pub fn all_pass(&self, node: &Node) -> bool {
        self.check_all(node)
            .iter()
            .all(|(_, result)| matches!(result, PropertyResult::Pass | PropertyResult::Skip(_)))
    }
}

impl Default for PropertySuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::Numeric;

    #[test]
    fn test_property_check() {
        let property = Property::new("always_pass", |_| PropertyResult::Pass);
        let node = Node::from("test");
        assert_eq!(property.check(&node), PropertyResult::Pass);
    }

    #[test]
    fn test_stringify_never_panics() {
        let node = Node::from("test");
        let result = properties::stringify_never_panics(&node);
        assert_eq!(result, PropertyResult::Pass);
    }

    #[test]
    fn test_depth_is_bounded() {
        let shallow = Node::from("test");
        assert_eq!(properties::depth_is_bounded(&shallow), PropertyResult::Pass);

        let nested = Node::Array(vec![Node::Array(vec![Node::Array(vec![Node::from(
            "deep",
        )])])]);
        assert_eq!(properties::depth_is_bounded(&nested), PropertyResult::Pass);
    }

    #[test]
    fn test_collections_are_finite() {
        let small = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
        assert_eq!(
            properties::collections_are_finite(&small),
            PropertyResult::Pass
        );
    }

    #[test]
    fn test_property_suite() {
        let suite = PropertySuite::common();
        let node = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::Number(Numeric::Integer(30))),
        ]);

        let results = suite.check_all(&node);
        assert!(results.len() > 0);

        // Most should pass
        let passed = results
            .iter()
            .filter(|(_, r)| matches!(r, PropertyResult::Pass))
            .count();
        assert!(passed > 0);
    }

    #[test]
    fn test_all_pass() {
        let suite = PropertySuite::common();
        let node = Node::from("simple");
        assert!(suite.all_pass(&node));
    }

    #[test]
    fn test_roundtrip_simple() {
        let node = Node::from("test");
        let result = properties::roundtrip_preserves_structure(&node);
        if !matches!(result, PropertyResult::Pass | PropertyResult::Skip(_)) {
            eprintln!("Roundtrip failed: {:?}", result);
        }
        assert!(matches!(
            result,
            PropertyResult::Pass | PropertyResult::Skip(_)
        ));
    }
}
