//! Performance measurement and optimization utilities
//!
//! Provides tools for profiling YAML operations and gathering statistics.

#[cfg(feature = "std")]
use std::time::{Duration, Instant};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::nodes::node::Node;

/// Statistics about a YAML document structure
#[derive(Clone, Debug, Default)]
pub struct DocumentStats {
    /// Total number of nodes in the document
    pub total_nodes: usize,

    /// Maximum nesting depth
    pub max_depth: usize,

    /// Number of string nodes
    pub string_count: usize,

    /// Number of numeric nodes
    pub number_count: usize,

    /// Number of boolean nodes
    pub boolean_count: usize,

    /// Number of array/sequence nodes
    pub array_count: usize,

    /// Number of mapping nodes
    pub mapping_count: usize,

    /// Number of set nodes
    pub set_count: usize,

    /// Number of anchor nodes
    pub anchor_count: usize,

    /// Number of alias nodes
    pub alias_count: usize,

    /// Number of tagged nodes
    pub tagged_count: usize,

    /// Total string length (bytes)
    pub total_string_bytes: usize,

    /// Largest array size
    pub largest_array: usize,

    /// Largest mapping size
    pub largest_mapping: usize,
}

impl DocumentStats {
    /// Create a new empty statistics object
    pub fn new() -> Self {
        Self::default()
    }

    /// Gather statistics from a Node tree
    ///
    /// # Example
    /// ```
    /// # use yaml_lib::{Node, DocumentStats};
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::from("text"),
    ///     Node::Array(vec![Node::from(2)])
    /// ]);
    ///
    /// let stats = DocumentStats::from_node(&doc);
    /// assert_eq!(stats.total_nodes, 5);
    /// assert_eq!(stats.max_depth, 2);
    /// assert_eq!(stats.array_count, 2);
    /// ```
    pub fn from_node(node: &Node) -> Self {
        let mut stats = Self::new();
        stats.analyze_node(node, 0);
        stats
    }

    fn analyze_node(&mut self, node: &Node, depth: usize) {
        self.total_nodes += 1;

        if depth > self.max_depth {
            self.max_depth = depth;
        }

        match node {
            Node::Str(s, _, _) => {
                self.string_count += 1;
                self.total_string_bytes += s.len();
            }
            Node::Number(_) => {
                self.number_count += 1;
            }
            Node::Boolean(_) => {
                self.boolean_count += 1;
            }
            Node::Array(items) => {
                self.array_count += 1;
                if items.len() > self.largest_array {
                    self.largest_array = items.len();
                }
                for item in items {
                    self.analyze_node(item, depth + 1);
                }
            }
            Node::Mapping(pairs) => {
                self.mapping_count += 1;
                if pairs.len() > self.largest_mapping {
                    self.largest_mapping = pairs.len();
                }
                for (key, value) in pairs {
                    self.analyze_node(key, depth + 1);
                    self.analyze_node(value, depth + 1);
                }
            }
            Node::Set(items) => {
                self.set_count += 1;
                for item in items {
                    self.analyze_node(item, depth + 1);
                }
            }
            Node::Document(nodes) | Node::Documents(nodes) => {
                for n in nodes {
                    self.analyze_node(n, depth + 1);
                }
            }
            Node::Anchored(inner, _) => {
                self.anchor_count += 1;
                self.analyze_node(inner, depth + 1);
            }
            Node::Tagged(inner, _) => {
                self.tagged_count += 1;
                self.analyze_node(inner, depth + 1);
            }
            Node::Alias(_) => {
                self.alias_count += 1;
            }
            Node::Comment(_) | Node::None => {}
        }
    }

    /// Calculate estimated memory usage in bytes
    pub fn estimated_memory_bytes(&self) -> usize {
        // Rough estimation based on typical sizes
        let node_overhead = self.total_nodes * 64; // ~64 bytes per Node enum
        let string_data = self.total_string_bytes;
        let collection_overhead = (self.array_count + self.mapping_count + self.set_count) * 24; // Vec overhead

        node_overhead + string_data + collection_overhead
    }

    /// Get a human-readable summary
    #[cfg(feature = "alloc")]
    pub fn summary(&self) -> String {
        alloc::format!(
            "Document Statistics:\n\
             - Total nodes: {}\n\
             - Max depth: {}\n\
             - Strings: {} ({} bytes)\n\
             - Numbers: {}\n\
             - Booleans: {}\n\
             - Arrays: {} (largest: {})\n\
             - Mappings: {} (largest: {})\n\
             - Sets: {}\n\
             - Anchors: {}\n\
             - Aliases: {}\n\
             - Tagged: {}\n\
             - Est. memory: {} bytes",
            self.total_nodes,
            self.max_depth,
            self.string_count,
            self.total_string_bytes,
            self.number_count,
            self.boolean_count,
            self.array_count,
            self.largest_array,
            self.mapping_count,
            self.largest_mapping,
            self.set_count,
            self.anchor_count,
            self.alias_count,
            self.tagged_count,
            self.estimated_memory_bytes()
        )
    }
}

/// Simple timer for measuring operation duration
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct Timer {
    start: Instant,
    label: String,
}

#[cfg(feature = "std")]
impl Timer {
    /// Start a new timer with a label
    pub fn new<S: Into<String>>(label: S) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    /// Get elapsed time since timer start
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and return elapsed duration
    pub fn stop(self) -> Duration {
        self.elapsed()
    }

    /// Stop the timer and print elapsed time (debug only)
    pub fn stop_and_print(self) {
        #[cfg(feature = "debug-trace")]
        {
            let elapsed = self.elapsed();
            println!("{}: {:?}", self.label, elapsed);
        }
    }
}

/// Performance profiler for measuring multiple operations
#[cfg(all(feature = "std", feature = "alloc"))]
#[derive(Debug, Default)]
pub struct Profiler {
    measurements: Vec<(String, Duration)>,
}

#[cfg(all(feature = "std", feature = "alloc"))]
impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }

    /// Time an operation and record it
    pub fn time<F, R>(&mut self, label: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        self.measurements.push((label.to_string(), elapsed));
        result
    }

    /// Get all measurements
    pub fn measurements(&self) -> &[(String, Duration)] {
        &self.measurements
    }

    /// Get total time across all measurements
    pub fn total_time(&self) -> Duration {
        self.measurements.iter().map(|(_, d)| *d).sum()
    }

    /// Print all measurements (debug only)
    pub fn print_results(&self) {
        #[cfg(feature = "debug-trace")]
        {
            println!("Performance Profile:");
            println!("{:-<60}", "");
            for (label, duration) in &self.measurements {
                println!("{:<40} {:>15?}", label, duration);
            }
            println!("{:-<60}", "");
            println!("{:<40} {:>15?}", "Total", self.total_time());
        }
    }

    /// Clear all measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
    }
}

/// Utility to compare performance of different approaches
#[cfg(feature = "std")]
pub fn compare_performance<F1, F2>(label1: &str, f1: F1, label2: &str, f2: F2)
where
    F1: FnOnce(),
    F2: FnOnce(),
{
    let timer1 = Timer::new(label1);
    f1();
    let time1 = timer1.stop();

    let timer2 = Timer::new(label2);
    f2();
    let time2 = timer2.stop();

    #[cfg(feature = "debug-trace")]
    println!("Performance Comparison:");
    #[cfg(feature = "debug-trace")]
    println!("  {}: {:?}", label1, time1);
    #[cfg(feature = "debug-trace")]
    println!("  {}: {:?}", label2, time2);

    if time1 < time2 {
        let _ratio = time2.as_secs_f64() / time1.as_secs_f64();
        #[cfg(feature = "debug-trace")]
        println!("  {} is {:.2}x faster", label1, _ratio);
    } else if time2 < time1 {
        let _ratio = time1.as_secs_f64() / time2.as_secs_f64();
        #[cfg(feature = "debug-trace")]
        println!("  {} is {:.2}x faster", label2, _ratio);
    } else {
        #[cfg(feature = "debug-trace")]
        println!("  Both approaches have similar performance");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_stats_empty() {
        let stats = DocumentStats::new();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.max_depth, 0);
    }

    #[test]
    fn test_document_stats_simple() {
        let node = Node::from(42);
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.total_nodes, 1);
        assert_eq!(stats.number_count, 1);
        assert_eq!(stats.max_depth, 0);
    }

    #[test]
    fn test_document_stats_array() {
        let node = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.total_nodes, 4); // array + 3 numbers
        assert_eq!(stats.array_count, 1);
        assert_eq!(stats.number_count, 3);
        assert_eq!(stats.largest_array, 3);
        assert_eq!(stats.max_depth, 1);
    }

    #[test]
    fn test_document_stats_nested() {
        let node = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
        ]);
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.total_nodes, 5);
        assert_eq!(stats.array_count, 2);
        assert_eq!(stats.number_count, 3);
        assert_eq!(stats.max_depth, 2);
    }

    #[test]
    fn test_document_stats_mapping() {
        let node = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
        ]);
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.total_nodes, 5); // mapping + 2 keys + 2 values
        assert_eq!(stats.mapping_count, 1);
        assert_eq!(stats.string_count, 4);
        assert_eq!(stats.largest_mapping, 2);
        assert_eq!(stats.total_string_bytes, 20); // "key1" + "value1" + "key2" + "value2"
    }

    #[test]
    fn test_document_stats_mixed() {
        let node = Node::Mapping(vec![
            (
                Node::from("numbers"),
                Node::Array(vec![Node::from(1), Node::from(2)]),
            ),
            (Node::from("text"), Node::from("hello")),
            (Node::from("flag"), Node::from(true)),
        ]);
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.mapping_count, 1);
        assert_eq!(stats.array_count, 1);
        assert_eq!(stats.string_count, 4); // 3 keys + 1 value
        assert_eq!(stats.number_count, 2);
        assert_eq!(stats.boolean_count, 1);
    }

    #[test]
    fn test_document_stats_anchors() {
        use alloc::boxed::Box;

        let node = Node::Anchored(Box::new(Node::from(42)), "anchor".to_string());
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.anchor_count, 1);
        assert_eq!(stats.number_count, 1);
    }

    #[test]
    fn test_document_stats_alias() {
        let node = Node::Alias("ref".to_string());
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.alias_count, 1);
        assert_eq!(stats.total_nodes, 1);
    }

    #[test]
    fn test_document_stats_tagged() {
        use alloc::boxed::Box;

        let node = Node::Tagged(Box::new(Node::from("value")), "!custom".to_string());
        let stats = DocumentStats::from_node(&node);

        assert_eq!(stats.tagged_count, 1);
        assert_eq!(stats.string_count, 1);
    }

    #[test]
    fn test_estimated_memory() {
        let node = Node::Array(vec![Node::from(1), Node::from(2)]);
        let stats = DocumentStats::from_node(&node);

        let mem = stats.estimated_memory_bytes();
        assert!(mem > 0);
        // Should account for nodes and collection overhead
        assert!(mem >= stats.total_nodes * 64);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_summary_format() {
        let node = Node::Array(vec![Node::from(1), Node::from(2)]);
        let stats = DocumentStats::from_node(&node);

        let summary = stats.summary();
        assert!(summary.contains("Total nodes: 3"));
        assert!(summary.contains("Arrays: 1"));
        assert!(summary.contains("Numbers: 2"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_timer_creation() {
        let timer = Timer::new("test");
        assert_eq!(timer.label, "test");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_timer_elapsed() {
        let timer = Timer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_timer_stop() {
        let timer = Timer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.stop();
        assert!(duration.as_millis() >= 10);
    }

    #[cfg(all(feature = "std", feature = "alloc"))]
    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new();
        assert_eq!(profiler.measurements.len(), 0);
    }

    #[cfg(all(feature = "std", feature = "alloc"))]
    #[test]
    fn test_profiler_time() {
        let mut profiler = Profiler::new();

        let result = profiler.time("test_op", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert_eq!(profiler.measurements().len(), 1);
        assert_eq!(profiler.measurements()[0].0, "test_op");
        assert!(profiler.measurements()[0].1.as_millis() >= 10);
    }

    #[cfg(all(feature = "std", feature = "alloc"))]
    #[test]
    fn test_profiler_multiple_measurements() {
        let mut profiler = Profiler::new();

        profiler.time("op1", || {
            std::thread::sleep(std::time::Duration::from_millis(10))
        });
        profiler.time("op2", || {
            std::thread::sleep(std::time::Duration::from_millis(20))
        });

        assert_eq!(profiler.measurements().len(), 2);
        let total = profiler.total_time();
        assert!(total.as_millis() >= 30);
    }

    #[cfg(all(feature = "std", feature = "alloc"))]
    #[test]
    fn test_profiler_clear() {
        let mut profiler = Profiler::new();

        profiler.time("op", || {});
        assert_eq!(profiler.measurements().len(), 1);

        profiler.clear();
        assert_eq!(profiler.measurements().len(), 0);
    }
}
