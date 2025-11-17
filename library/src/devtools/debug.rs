//! Debug utilities for YAML development
//!
//! Provides debugging tools including breakpoints, logging, and trace capabilities.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::nodes::node::Node;

/// Debug level for controlling output verbosity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

/// Debug context for tracking execution flow
#[derive(Debug, Clone)]
pub struct DebugContext {
    level: DebugLevel,
    logs: Vec<String>,
    breakpoints: Vec<String>,
}

impl DebugContext {
    /// Create a new debug context
    pub fn new(level: DebugLevel) -> Self {
        Self {
            level,
            logs: Vec::new(),
            breakpoints: Vec::new(),
        }
    }

    /// Create with info level
    pub fn with_info() -> Self {
        Self::new(DebugLevel::Info)
    }

    /// Create with debug level
    pub fn with_debug() -> Self {
        Self::new(DebugLevel::Debug)
    }

    /// Create with trace level
    pub fn with_trace() -> Self {
        Self::new(DebugLevel::Trace)
    }

    /// Set debug level
    pub fn set_level(&mut self, level: DebugLevel) {
        self.level = level;
    }

    /// Get current debug level
    pub fn level(&self) -> DebugLevel {
        self.level
    }

    /// Log a message at the specified level
    pub fn log(&mut self, level: DebugLevel, message: String) {
        if level <= self.level {
            self.logs.push(format!("[{:?}] {}", level, message));
        }
    }

    /// Log error message
    pub fn error(&mut self, message: String) {
        self.log(DebugLevel::Error, message);
    }

    /// Log warning message
    pub fn warn(&mut self, message: String) {
        self.log(DebugLevel::Warn, message);
    }

    /// Log info message
    pub fn info(&mut self, message: String) {
        self.log(DebugLevel::Info, message);
    }

    /// Log debug message
    pub fn debug(&mut self, message: String) {
        self.log(DebugLevel::Debug, message);
    }

    /// Log trace message
    pub fn trace(&mut self, message: String) {
        self.log(DebugLevel::Trace, message);
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, name: String) {
        self.breakpoints.push(name);
    }

    /// Check if a breakpoint exists
    pub fn has_breakpoint(&self, name: &str) -> bool {
        self.breakpoints.iter().any(|bp| bp == name)
    }

    /// Get all logs
    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    /// Get all breakpoints
    pub fn breakpoints(&self) -> &[String] {
        &self.breakpoints
    }

    /// Clear all logs
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    /// Format all logs as a string
    pub fn format_logs(&self) -> String {
        self.logs.join("\n")
    }
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new(DebugLevel::Info)
    }
}

/// Node debugger for inspecting node operations
pub struct NodeDebugger {
    context: DebugContext,
    trace_enabled: bool,
}

impl NodeDebugger {
    /// Create a new node debugger
    pub fn new() -> Self {
        Self {
            context: DebugContext::with_debug(),
            trace_enabled: false,
        }
    }

    /// Create with tracing enabled
    pub fn with_trace() -> Self {
        Self {
            context: DebugContext::with_trace(),
            trace_enabled: true,
        }
    }

    /// Enable tracing
    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
        self.context.set_level(DebugLevel::Trace);
    }

    /// Disable tracing
    pub fn disable_trace(&mut self) {
        self.trace_enabled = false;
    }

    /// Get the debug context
    pub fn context(&self) -> &DebugContext {
        &self.context
    }

    /// Get mutable debug context
    pub fn context_mut(&mut self) -> &mut DebugContext {
        &mut self.context
    }

    /// Debug a node access operation
    pub fn debug_access(&mut self, path: &str, node: &Node) {
        let msg = format!(
            "Access: {} -> {:?}",
            path,
            crate::devtools::inspect::node_type(node)
        );
        self.context.debug(msg);
    }

    /// Debug a node creation operation
    pub fn debug_create(&mut self, node: &Node) {
        let msg = format!("Create: {:?}", crate::devtools::inspect::node_type(node));
        self.context.debug(msg);
    }

    /// Debug a node modification operation
    pub fn debug_modify(&mut self, path: &str, old: &Node, new: &Node) {
        let msg = format!(
            "Modify: {} from {:?} to {:?}",
            path,
            crate::devtools::inspect::node_type(old),
            crate::devtools::inspect::node_type(new)
        );
        self.context.debug(msg);
    }

    /// Trace node traversal
    pub fn trace_visit(&mut self, depth: usize, node: &Node) {
        if self.trace_enabled {
            let indent = "  ".repeat(depth);
            let msg = format!(
                "{}Visit: {:?}",
                indent,
                crate::devtools::inspect::node_type(node)
            );
            self.context.trace(msg);
        }
    }

    /// Get formatted logs
    pub fn logs(&self) -> String {
        self.context.format_logs()
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.context.clear_logs();
        self.context.clear_breakpoints();
    }
}

impl Default for NodeDebugger {
    fn default() -> Self {
        Self::new()
    }
}

/// Assertion helper for debugging
pub struct DebugAssert;

impl DebugAssert {
    /// Assert node type
    pub fn assert_type(
        node: &Node,
        expected: crate::devtools::inspect::NodeType,
    ) -> Result<(), String> {
        let actual = crate::devtools::inspect::node_type(node);
        if actual == expected {
            Ok(())
        } else {
            Err(format!(
                "Type assertion failed: expected {:?}, got {:?}",
                expected, actual
            ))
        }
    }

    /// Assert node has specific size
    pub fn assert_size(node: &Node, expected: usize) -> Result<(), String> {
        let actual = crate::devtools::inspect::node_size(node);
        if actual == expected {
            Ok(())
        } else {
            Err(format!(
                "Size assertion failed: expected {}, got {}",
                expected, actual
            ))
        }
    }

    /// Assert node has maximum depth
    pub fn assert_max_depth(node: &Node, max_depth: usize) -> Result<(), String> {
        let actual = crate::devtools::inspect::node_depth(node);
        if actual <= max_depth {
            Ok(())
        } else {
            Err(format!(
                "Depth assertion failed: expected <= {}, got {}",
                max_depth, actual
            ))
        }
    }

    /// Assert node is scalar
    pub fn assert_scalar(node: &Node) -> Result<(), String> {
        let node_type = crate::devtools::inspect::node_type(node);
        if node_type.is_scalar() {
            Ok(())
        } else {
            Err(format!("Scalar assertion failed: node is {:?}", node_type))
        }
    }

    /// Assert node is collection
    pub fn assert_collection(node: &Node) -> Result<(), String> {
        let node_type = crate::devtools::inspect::node_type(node);
        if node_type.is_collection() {
            Ok(())
        } else {
            Err(format!(
                "Collection assertion failed: node is {:?}",
                node_type
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_level_ordering() {
        assert!(DebugLevel::Error < DebugLevel::Warn);
        assert!(DebugLevel::Warn < DebugLevel::Info);
        assert!(DebugLevel::Info < DebugLevel::Debug);
        assert!(DebugLevel::Debug < DebugLevel::Trace);
    }

    #[test]
    fn test_debug_context() {
        let mut ctx = DebugContext::new(DebugLevel::Info);

        ctx.error("Error message".to_string());
        ctx.info("Info message".to_string());
        ctx.debug("Debug message".to_string());

        let logs = ctx.logs();
        assert_eq!(logs.len(), 2); // Error and Info, but not Debug
        assert!(logs[0].contains("Error"));
        assert!(logs[1].contains("Info"));
    }

    #[test]
    fn test_breakpoints() {
        let mut ctx = DebugContext::new(DebugLevel::Debug);

        ctx.add_breakpoint("parse_start".to_string());
        ctx.add_breakpoint("parse_end".to_string());

        assert!(ctx.has_breakpoint("parse_start"));
        assert!(ctx.has_breakpoint("parse_end"));
        assert!(!ctx.has_breakpoint("unknown"));

        ctx.clear_breakpoints();
        assert!(!ctx.has_breakpoint("parse_start"));
    }

    #[test]
    fn test_node_debugger() {
        let mut debugger = NodeDebugger::new();
        let node = Node::from("test");

        debugger.debug_create(&node);
        debugger.debug_access("/path/to/node", &node);

        let logs = debugger.logs();
        assert!(logs.contains("Create"));
        assert!(logs.contains("Access"));
    }

    #[test]
    fn test_debug_assert_type() {
        use crate::devtools::inspect::NodeType;

        let node = Node::from("test");
        assert!(DebugAssert::assert_type(&node, NodeType::String).is_ok());
        assert!(DebugAssert::assert_type(&node, NodeType::Integer).is_err());
    }

    #[test]
    fn test_debug_assert_size() {
        let node = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert!(DebugAssert::assert_size(&node, 2).is_ok());
        assert!(DebugAssert::assert_size(&node, 3).is_err());
    }

    #[test]
    fn test_debug_assert_scalar() {
        assert!(DebugAssert::assert_scalar(&Node::from("test")).is_ok());
        assert!(DebugAssert::assert_scalar(&Node::Array(vec![])).is_err());
    }

    #[test]
    fn test_trace_enabled() {
        let mut debugger = NodeDebugger::with_trace();
        assert!(debugger.trace_enabled);

        debugger.disable_trace();
        assert!(!debugger.trace_enabled);
    }
}
