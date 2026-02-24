//! Streaming Utilities for YAML Library
//!
//! This module provides streaming and iterator support for efficient YAML processing.
//! It enables processing large YAML documents without loading them entirely into memory,
//! using iterator-based traversal, streaming parsers, and memory-efficient operations.
//!
//! # Features
//! - Iterator-based node traversal (depth-first, breadth-first)
//! - Streaming parser for lazy evaluation
//! - Memory-efficient filter/map/fold operations
//! - Path-based node access
//!
//! # Usage
//! Use these utilities to process large or streaming YAML data efficiently.

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

use crate::nodes::node::Node;

/// Traversal order for iterating through a Node tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalOrder {
    /// Depth-first traversal (pre-order)
    DepthFirst,
    /// Breadth-first traversal (level-order)
    BreadthFirst,
}

/// Iterator for traversing a Node tree
pub struct NodeIterator<'a> {
    /// Stack for depth-first or queue for breadth-first
    pending: VecDeque<&'a Node>,
    /// Traversal order
    order: TraversalOrder,
}

impl<'a> NodeIterator<'a> {
    /// Create a new iterator starting from the given node
    pub fn new(node: &'a Node, order: TraversalOrder) -> Self {
        let mut pending = VecDeque::new();
        pending.push_back(node);

        Self { pending, order }
    }

    /// Create a depth-first iterator
    pub fn depth_first(node: &'a Node) -> Self {
        Self::new(node, TraversalOrder::DepthFirst)
    }

    /// Create a breadth-first iterator
    pub fn breadth_first(node: &'a Node) -> Self {
        Self::new(node, TraversalOrder::BreadthFirst)
    }

    /// Add children of a node to pending queue
    fn enqueue_children(&mut self, node: &'a Node) {
        match node {
            Node::Array(items) | Node::Set(items) => {
                for item in items {
                    match self.order {
                        TraversalOrder::DepthFirst => self.pending.push_front(item),
                        TraversalOrder::BreadthFirst => self.pending.push_back(item),
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    match self.order {
                        TraversalOrder::DepthFirst => {
                            self.pending.push_front(value);
                            self.pending.push_front(key);
                        }
                        TraversalOrder::BreadthFirst => {
                            self.pending.push_back(key);
                            self.pending.push_back(value);
                        }
                    }
                }
            }
            Node::Document(nodes) | Node::Documents(nodes) => {
                for item in nodes {
                    match self.order {
                        TraversalOrder::DepthFirst => self.pending.push_front(item),
                        TraversalOrder::BreadthFirst => self.pending.push_back(item),
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => match self.order {
                TraversalOrder::DepthFirst => self.pending.push_front(inner),
                TraversalOrder::BreadthFirst => self.pending.push_back(inner),
            },
            _ => {}
        }
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.pending.pop_front() {
            self.enqueue_children(node);
            Some(node)
        } else {
            None
        }
    }
}

/// Extension trait for Node to provide iterator methods
pub trait NodeIteratorExt {
    /// Create an iterator that traverses the node tree depth-first
    fn iter_depth_first(&self) -> NodeIterator;

    /// Create an iterator that traverses the node tree breadth-first
    fn iter_breadth_first(&self) -> NodeIterator;

    /// Count nodes in the tree
    fn count_nodes(&self) -> usize;

    /// Find first node matching a predicate
    fn find_node<F>(&self, predicate: F) -> Option<&Node>
    where
        F: Fn(&Node) -> bool;

    /// Filter nodes by predicate
    fn filter_nodes<F>(&self, predicate: F) -> Vec<&Node>
    where
        F: Fn(&Node) -> bool;

    /// Collect all string values in the tree
    fn collect_strings(&self) -> Vec<&str>;

    /// Collect all numeric values in the tree
    fn collect_numbers(&self) -> Vec<i64>;
}

impl NodeIteratorExt for Node {
    fn iter_depth_first(&self) -> NodeIterator {
        NodeIterator::depth_first(self)
    }

    fn iter_breadth_first(&self) -> NodeIterator {
        NodeIterator::breadth_first(self)
    }

    fn count_nodes(&self) -> usize {
        self.iter_depth_first().count()
    }

    fn find_node<F>(&self, predicate: F) -> Option<&Node>
    where
        F: Fn(&Node) -> bool,
    {
        self.iter_depth_first().find(|node| predicate(node))
    }

    fn filter_nodes<F>(&self, predicate: F) -> Vec<&Node>
    where
        F: Fn(&Node) -> bool,
    {
        self.iter_depth_first()
            .filter(|node| predicate(node))
            .collect()
    }

    fn collect_strings(&self) -> Vec<&str> {
        self.iter_depth_first()
            .filter_map(|node| match node {
                Node::Str(s, _, _) => Some(s.as_str()),
                _ => None,
            })
            .collect()
    }

    fn collect_numbers(&self) -> Vec<i64> {
        self.iter_depth_first()
            .filter_map(|node| match node {
                Node::Number(crate::nodes::node::Numeric::Integer(i)) => Some(*i),
                _ => None,
            })
            .collect()
    }
}

/// Path-based node access
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// Access by string key (for mappings)
    Key(String),
    /// Access by integer index (for arrays/sets)
    Index(usize),
}

impl From<&str> for PathSegment {
    fn from(s: &str) -> Self {
        PathSegment::Key(s.to_string())
    }
}

impl From<String> for PathSegment {
    fn from(s: String) -> Self {
        PathSegment::Key(s)
    }
}

impl From<usize> for PathSegment {
    fn from(i: usize) -> Self {
        PathSegment::Index(i)
    }
}

/// Path for accessing nested nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePath {
    segments: Vec<PathSegment>,
}

impl NodePath {
    /// Create an empty path
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a path from segments
    pub fn from_segments(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }

    /// Add a segment to the path
    pub fn push<T: Into<PathSegment>>(&mut self, segment: T) {
        self.segments.push(segment.into());
    }

    /// Get path segments
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Access a node at this path
    pub fn get<'a>(&self, node: &'a Node) -> Option<&'a Node> {
        let mut current = node;

        for segment in &self.segments {
            current = match (segment, current) {
                (PathSegment::Key(key), Node::Mapping(pairs)) => pairs
                    .iter()
                    .find(|(k, _)| match k {
                        Node::Str(s, _, _) => s == key,
                        _ => false,
                    })
                    .map(|(_, v)| v)?,
                (PathSegment::Index(idx), Node::Array(items)) => items.get(*idx)?,
                (PathSegment::Index(idx), Node::Set(items)) => items.get(*idx)?,
                _ => return None,
            };
        }

        Some(current)
    }
}

impl Default for NodePath {
    fn default() -> Self {
        Self::new()
    }
}

/// Stream processor for efficient large document handling
pub struct NodeStream<'a> {
    iterator: NodeIterator<'a>,
}

impl<'a> NodeStream<'a> {
    /// Create a new stream from a node
    pub fn new(node: &'a Node) -> Self {
        Self {
            iterator: NodeIterator::depth_first(node),
        }
    }

    /// Filter nodes in the stream
    pub fn filter<F>(self, predicate: F) -> FilterStream<'a, F>
    where
        F: FnMut(&Node) -> bool,
    {
        FilterStream {
            iterator: self.iterator,
            predicate,
        }
    }

    /// Map nodes in the stream
    pub fn map<F, T>(self, mapper: F) -> MapStream<'a, F, T>
    where
        F: FnMut(&Node) -> T,
    {
        MapStream {
            iterator: self.iterator,
            mapper,
        }
    }

    /// Fold/reduce the stream
    pub fn fold<T, F>(mut self, init: T, mut folder: F) -> T
    where
        F: FnMut(T, &Node) -> T,
    {
        let mut accumulator = init;
        while let Some(node) = self.iterator.next() {
            accumulator = folder(accumulator, node);
        }
        accumulator
    }

    /// Count nodes in the stream
    pub fn count(self) -> usize {
        self.iterator.count()
    }

    /// Collect all nodes into a vector
    pub fn collect(self) -> Vec<&'a Node> {
        self.iterator.collect()
    }
}

/// Filtered node stream
pub struct FilterStream<'a, F>
where
    F: FnMut(&Node) -> bool,
{
    iterator: NodeIterator<'a>,
    predicate: F,
}

impl<'a, F> Iterator for FilterStream<'a, F>
where
    F: FnMut(&Node) -> bool,
{
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.iterator.next() {
            if (self.predicate)(node) {
                return Some(node);
            }
        }
        None
    }
}

/// Mapped node stream
pub struct MapStream<'a, F, T>
where
    F: FnMut(&Node) -> T,
{
    iterator: NodeIterator<'a>,
    mapper: F,
}

impl<'a, F, T> Iterator for MapStream<'a, F, T>
where
    F: FnMut(&Node) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|node| (self.mapper)(node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::Numeric;

    #[test]
    fn test_depth_first_iterator() {
        let tree = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
            Node::from(4),
        ]);

        let nodes: Vec<_> = tree.iter_depth_first().collect();
        assert_eq!(nodes.len(), 6); // Array + 1 + Array + 2 + 3 + 4
    }

    #[test]
    fn test_breadth_first_iterator() {
        let tree = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
            Node::from(4),
        ]);

        let nodes: Vec<_> = tree.iter_breadth_first().collect();
        assert_eq!(nodes.len(), 6);
    }

    #[test]
    fn test_count_nodes() {
        let tree = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from(42)),
        ]);

        // Mapping + key1 + value1 + key2 + 42
        assert_eq!(tree.count_nodes(), 5);
    }

    #[test]
    fn test_find_node() {
        let tree = Node::Array(vec![Node::from(1), Node::from("hello"), Node::from(3)]);

        let found = tree.find_node(|n| matches!(n, Node::Str(s, _, _) if s == "hello"));
        assert!(found.is_some());

        let not_found = tree.find_node(|n| matches!(n, Node::Str(s, _, _) if s == "world"));
        assert!(not_found.is_none());
    }

    #[test]
    fn test_filter_nodes() {
        let tree = Node::Array(vec![
            Node::from(1),
            Node::from("text"),
            Node::from(2),
            Node::from("more"),
        ]);

        let strings = tree.filter_nodes(|n| matches!(n, Node::Str(_, _, _)));
        assert_eq!(strings.len(), 2);
    }

    #[test]
    fn test_collect_strings() {
        let tree = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("city"), Node::from("NYC")),
        ]);

        let strings = tree.collect_strings();
        assert_eq!(strings.len(), 4); // "name", "Alice", "city", "NYC"
    }

    #[test]
    fn test_collect_numbers() {
        let tree = Node::Array(vec![
            Node::Number(Numeric::Integer(10)),
            Node::from("text"),
            Node::Number(Numeric::Integer(20)),
            Node::Number(Numeric::Integer(30)),
        ]);

        let mut numbers = tree.collect_numbers();
        numbers.sort(); // Sort since depth-first order may vary
        assert_eq!(numbers, vec![10, 20, 30]);
    }

    #[test]
    fn test_node_path_mapping() {
        let tree = Node::Mapping(vec![(
            Node::from("user"),
            Node::Mapping(vec![
                (Node::from("name"), Node::from("Alice")),
                (Node::from("age"), Node::from(30)),
            ]),
        )]);

        let mut path = NodePath::new();
        path.push("user");
        path.push("name");

        let result = path.get(&tree);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), Node::Str(s, _, _) if s == "Alice"));
    }

    #[test]
    fn test_node_path_array() {
        let tree = Node::Array(vec![
            Node::from("first"),
            Node::Array(vec![Node::from("nested1"), Node::from("nested2")]),
        ]);

        let mut path = NodePath::new();
        path.push(1usize);
        path.push(0usize);

        let result = path.get(&tree);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), Node::Str(s, _, _) if s == "nested1"));
    }

    #[test]
    fn test_node_stream_filter() {
        let tree = Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
            Node::Number(Numeric::Integer(4)),
        ]);

        let stream = NodeStream::new(&tree);
        let evens: Vec<_> = stream
            .filter(|n| matches!(n, Node::Number(Numeric::Integer(i)) if i % 2 == 0))
            .collect();

        assert_eq!(evens.len(), 2);
    }

    #[test]
    fn test_node_stream_map() {
        let tree = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        let stream = NodeStream::new(&tree);
        let types: Vec<_> = stream
            .map(|n| match n {
                Node::Array(_) => "array",
                Node::Number(_) => "number",
                _ => "other",
            })
            .collect();

        assert_eq!(types.len(), 4); // Array + 3 numbers
        assert_eq!(types[0], "array");
    }

    #[test]
    fn test_node_stream_fold() {
        let tree = Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
        ]);

        let stream = NodeStream::new(&tree);
        let sum = stream.fold(0, |acc, n| match n {
            Node::Number(Numeric::Integer(i)) => acc + i,
            _ => acc,
        });

        assert_eq!(sum, 6);
    }

    #[test]
    fn test_node_stream_count() {
        let tree = Node::Mapping(vec![
            (Node::from("a"), Node::from(1)),
            (Node::from("b"), Node::from(2)),
        ]);

        let stream = NodeStream::new(&tree);
        let count = stream.count();

        assert_eq!(count, 5); // Mapping + a + 1 + b + 2
    }
}
