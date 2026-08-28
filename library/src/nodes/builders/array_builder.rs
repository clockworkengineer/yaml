//! Array Node Fluent Builder
//!
//! Provides a chainable builder for constructing YAML Array nodes.

use crate::nodes::node::Node;

/// Builder for constructing Array nodes with a fluent API
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
