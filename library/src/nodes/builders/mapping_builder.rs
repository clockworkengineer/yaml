//! Mapping Node Fluent Builder
//!
//! Provides a chainable builder for constructing YAML Mapping nodes.

use crate::nodes::node::Node;

/// Builder for constructing Mapping nodes with a fluent API
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
