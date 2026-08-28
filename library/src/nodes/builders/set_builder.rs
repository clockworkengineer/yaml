//! Set Node Fluent Builder
//!
//! Provides a chainable builder for constructing YAML Set nodes.

use crate::nodes::node::Node;

/// Builder for constructing Set nodes with a fluent API
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
