//! Execution tracing and profiling
//!
//! Provides tools for tracing execution flow and measuring performance.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::time::Instant;

/// Trace event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEvent {
    Enter,
    Exit,
    Call,
    Return,
    Error,
    Custom(String),
}

/// A single trace entry
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub event: TraceEvent,
    pub location: String,
    pub message: String,
    pub depth: usize,
    #[cfg(feature = "std")]
    pub timestamp: Option<Instant>,
}

impl TraceEntry {
    /// Create a new trace entry
    pub fn new(event: TraceEvent, location: String, message: String, depth: usize) -> Self {
        Self {
            event,
            location,
            message,
            depth,
            #[cfg(feature = "std")]
            timestamp: Some(Instant::now()),
        }
    }

    /// Format as a readable string
    pub fn format(&self) -> String {
        let indent = "  ".repeat(self.depth);
        format!(
            "{}{:?} @ {}: {}",
            indent, self.event, self.location, self.message
        )
    }
}

/// Execution tracer
pub struct Tracer {
    entries: Vec<TraceEntry>,
    depth: usize,
    enabled: bool,
}

impl Tracer {
    /// Create a new tracer
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            depth: 0,
            enabled: true,
        }
    }

    /// Enable tracing
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable tracing
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enter a scope
    pub fn enter(&mut self, location: String, message: String) {
        if self.enabled {
            self.entries.push(TraceEntry::new(
                TraceEvent::Enter,
                location,
                message,
                self.depth,
            ));
            self.depth += 1;
        }
    }

    /// Exit a scope
    pub fn exit(&mut self, location: String, message: String) {
        if self.enabled {
            if self.depth > 0 {
                self.depth -= 1;
            }
            self.entries.push(TraceEntry::new(
                TraceEvent::Exit,
                location,
                message,
                self.depth,
            ));
        }
    }

    /// Log a function call
    pub fn call(&mut self, location: String, function: String) {
        if self.enabled {
            self.entries.push(TraceEntry::new(
                TraceEvent::Call,
                location,
                format!("Call: {}", function),
                self.depth,
            ));
        }
    }

    /// Log a function return
    pub fn return_value(&mut self, location: String, value: String) {
        if self.enabled {
            self.entries.push(TraceEntry::new(
                TraceEvent::Return,
                location,
                format!("Return: {}", value),
                self.depth,
            ));
        }
    }

    /// Log an error
    pub fn error(&mut self, location: String, error: String) {
        if self.enabled {
            self.entries.push(TraceEntry::new(
                TraceEvent::Error,
                location,
                format!("Error: {}", error),
                self.depth,
            ));
        }
    }

    /// Log a custom event
    pub fn custom(&mut self, event: String, location: String, message: String) {
        if self.enabled {
            self.entries.push(TraceEntry::new(
                TraceEvent::Custom(event),
                location,
                message,
                self.depth,
            ));
        }
    }

    /// Get all trace entries
    pub fn entries(&self) -> &[TraceEntry] {
        &self.entries
    }

    /// Get number of entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.depth = 0;
    }

    /// Format all entries as a string
    pub fn format(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.format())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get entries of a specific type
    pub fn filter_by_event(&self, event_type: TraceEvent) -> Vec<&TraceEntry> {
        self.entries
            .iter()
            .filter(|e| e.event == event_type)
            .collect()
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped trace guard that automatically exits on drop
pub struct TraceGuard<'a> {
    tracer: &'a mut Tracer,
    location: String,
}

impl<'a> TraceGuard<'a> {
    /// Create a new trace guard
    pub fn new(tracer: &'a mut Tracer, location: String, message: String) -> Self {
        tracer.enter(location.clone(), message);
        Self { tracer, location }
    }
}

impl<'a> Drop for TraceGuard<'a> {
    fn drop(&mut self) {
        self.tracer
            .exit(self.location.clone(), "scope exit".to_string());
    }
}

/// Performance measurement for traced operations
#[cfg(feature = "std")]
pub struct TracedTimer {
    start: Instant,
    location: String,
}

#[cfg(feature = "std")]
impl TracedTimer {
    /// Start a new timer
    pub fn start(location: String) -> Self {
        Self {
            start: Instant::now(),
            location,
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// Format timing information
    pub fn format(&self) -> String {
        format!("{}: {}ms", self.location, self.elapsed_ms())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_basic() {
        let mut tracer = Tracer::new();

        tracer.enter("main".to_string(), "Starting".to_string());
        tracer.call("main".to_string(), "process()".to_string());
        tracer.exit("main".to_string(), "Done".to_string());

        assert_eq!(tracer.count(), 3);
        assert_eq!(tracer.entries()[0].event, TraceEvent::Enter);
        assert_eq!(tracer.entries()[1].event, TraceEvent::Call);
        assert_eq!(tracer.entries()[2].event, TraceEvent::Exit);
    }

    #[test]
    fn test_tracer_depth() {
        let mut tracer = Tracer::new();

        tracer.enter("outer".to_string(), "Outer".to_string());
        assert_eq!(tracer.depth, 1);

        tracer.enter("inner".to_string(), "Inner".to_string());
        assert_eq!(tracer.depth, 2);

        tracer.exit("inner".to_string(), "Done".to_string());
        assert_eq!(tracer.depth, 1);

        tracer.exit("outer".to_string(), "Done".to_string());
        assert_eq!(tracer.depth, 0);
    }

    #[test]
    fn test_tracer_enable_disable() {
        let mut tracer = Tracer::new();

        tracer.enter("test".to_string(), "Message".to_string());
        assert_eq!(tracer.count(), 1);

        tracer.disable();
        tracer.enter("test2".to_string(), "Message2".to_string());
        assert_eq!(tracer.count(), 1); // Should not increase

        tracer.enable();
        tracer.enter("test3".to_string(), "Message3".to_string());
        assert_eq!(tracer.count(), 2);
    }

    #[test]
    fn test_filter_by_event() {
        let mut tracer = Tracer::new();

        tracer.enter("loc1".to_string(), "msg".to_string());
        tracer.call("loc2".to_string(), "func".to_string());
        tracer.error("loc3".to_string(), "err".to_string());
        tracer.exit("loc4".to_string(), "done".to_string());

        let errors = tracer.filter_by_event(TraceEvent::Error);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("err"));
    }

    #[test]
    fn test_trace_guard() {
        let mut tracer = Tracer::new();

        {
            let _guard = TraceGuard::new(&mut tracer, "scope".to_string(), "enter".to_string());
            // Can't check tracer state while guard is active due to borrow
        } // Guard drops here

        assert_eq!(tracer.count(), 2);
        assert_eq!(tracer.depth, 0);
        assert_eq!(tracer.entries()[1].event, TraceEvent::Exit);
    }

    #[test]
    fn test_clear() {
        let mut tracer = Tracer::new();

        tracer.enter("test".to_string(), "msg".to_string());
        tracer.exit("test".to_string(), "done".to_string());
        assert_eq!(tracer.count(), 2);

        tracer.clear();
        assert_eq!(tracer.count(), 0);
        assert_eq!(tracer.depth, 0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_traced_timer() {
        let timer = TracedTimer::start("test_operation".to_string());

        // Simulate some work
        let _ = (0..1000).sum::<i32>();

        let _elapsed = timer.elapsed_ms();
        // elapsed is always >= 0 for u128, no need to assert

        let formatted = timer.format();
        assert!(formatted.contains("test_operation"));
    }
}
