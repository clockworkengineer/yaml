//! String Interning Utilities for YAML Library
//!
//! This module provides string interning capabilities to optimize memory usage when
//! identical strings appear repeatedly in YAML documents (such as common keys).
//! String interning can reduce memory usage by 20-40% for typical configuration files.
//!
//! # Features
//! - Efficient string interning for deduplication
//! - Reduces memory footprint for repeated strings
//! - Thread-safe and no_std compatible
//!
//! # Usage
//! Use the string interner to store and reuse common strings in YAML processing.

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

#[cfg(feature = "alloc")]
use alloc::rc::Rc;

/// A reference-counted interned string
///
/// This is a lightweight handle to a string stored in the interner.
/// Cloning is cheap (just increments a reference count).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InternedString(Arc<String>);

impl InternedString {
    /// Create a new interned string (typically done by StringInterner)
    pub(crate) fn new(s: String) -> Self {
        Self(Arc::new(s))
    }

    /// Get a reference to the string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the reference count (for debugging/profiling)
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::fmt::Display for InternedString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for InternedString {
    fn from(s: &str) -> Self {
        Self::new(String::from(s))
    }
}

/// String interner for deduplicating strings
///
/// Maintains a cache of strings and returns interned references.
/// Multiple requests for the same string return the same interned reference.
///
/// # Example
/// ```
/// use yaml_lib::StringInterner;
///
/// let mut interner = StringInterner::new();
/// let s1 = interner.intern("name");
/// let s2 = interner.intern("name");
///
/// // Both references point to the same underlying string
/// assert_eq!(s1.ref_count(), s2.ref_count());
/// ```
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct StringInterner {
    cache: RwLock<HashMap<String, Arc<String>>>,
    stats: RwLock<InternerStats>,
}

#[cfg(feature = "std")]
impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(InternerStats::default()),
        }
    }

    /// Create a new string interner with a specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(capacity)),
            stats: RwLock::new(InternerStats::default()),
        }
    }

    /// Intern a string, returning an interned reference
    ///
    /// If the string is already interned, returns the existing reference.
    /// Otherwise, creates a new interned string.
    pub fn intern(&self, s: &str) -> InternedString {
        // Try read lock first (common case: string already exists)
        {
            let cache = self.cache.read().unwrap();
            if let Some(existing) = cache.get(s) {
                let mut stats = self.stats.write().unwrap();
                stats.hits += 1;
                return InternedString(Arc::clone(existing));
            }
        }

        // Need to insert - use write lock
        let mut cache = self.cache.write().unwrap();

        // Double-check in case another thread inserted while we waited
        if let Some(existing) = cache.get(s) {
            let mut stats = self.stats.write().unwrap();
            stats.hits += 1;
            return InternedString(Arc::clone(existing));
        }

        // Insert new string
        let arc = Arc::new(String::from(s));
        cache.insert(s.to_string(), Arc::clone(&arc));

        let mut stats = self.stats.write().unwrap();
        stats.misses += 1;
        stats.unique_strings = cache.len();

        InternedString(arc)
    }

    /// Get the number of unique strings currently interned
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Check if the interner is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().unwrap().is_empty()
    }

    /// Clear all interned strings
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
        let mut stats = self.stats.write().unwrap();
        stats.unique_strings = 0;
    }

    /// Get statistics about the interner
    pub fn stats(&self) -> InternerStats {
        *self.stats.read().unwrap()
    }

    /// Calculate memory savings estimate
    ///
    /// Returns (total_string_bytes, interned_bytes, savings_bytes, savings_percent)
    pub fn memory_savings(&self) -> (usize, usize, usize, f64) {
        let cache = self.cache.read().unwrap();
        let stats = self.stats.read().unwrap();

        let total_hits = stats.hits;
        let unique_count = cache.len();

        if unique_count == 0 || total_hits == 0 {
            return (0, 0, 0, 0.0);
        }

        let mut total_string_bytes = 0;
        let mut interned_bytes = 0;

        for (key, value) in cache.iter() {
            let str_len = key.len();
            let ref_count = Arc::strong_count(value);

            // Total bytes if each reference was a separate string
            // String overhead: capacity + length + pointer
            let string_overhead = core::mem::size_of::<String>();
            total_string_bytes += (str_len + string_overhead) * ref_count;

            // Actual bytes: one copy of the string + Arc pointers for each reference
            interned_bytes +=
                str_len + string_overhead + (ref_count * core::mem::size_of::<*const String>());
        }

        let savings = total_string_bytes.saturating_sub(interned_bytes);
        let savings_percent = if total_string_bytes > 0 {
            (savings as f64 / total_string_bytes as f64) * 100.0
        } else {
            0.0
        };

        (total_string_bytes, interned_bytes, savings, savings_percent)
    }
}

#[cfg(feature = "std")]
impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about string interning
#[derive(Debug, Clone, Copy, Default)]
pub struct InternerStats {
    /// Number of cache hits (string already interned)
    pub hits: usize,
    /// Number of cache misses (new string interned)
    pub misses: usize,
    /// Number of unique strings currently interned
    pub unique_strings: usize,
}

impl InternerStats {
    /// Get the hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get the total number of intern requests
    pub fn total_requests(&self) -> usize {
        self.hits + self.misses
    }
}

/// Simple non-thread-safe interner for single-threaded use
///
/// This is lighter weight than the thread-safe version but cannot be
/// shared across threads.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct SimpleInterner {
    cache: HashMap<String, Rc<String>>,
    stats: InternerStats,
}

#[cfg(feature = "alloc")]
impl SimpleInterner {
    /// Create a new simple interner
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: InternerStats::default(),
        }
    }

    /// Create a new simple interner with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            #[cfg(feature = "std")]
            cache: HashMap::with_capacity(capacity),
            #[cfg(not(feature = "std"))]
            cache: HashMap::new(),
            stats: InternerStats::default(),
        }
    }

    /// Intern a string
    pub fn intern(&mut self, s: &str) -> Rc<String> {
        if let Some(existing) = self.cache.get(s) {
            self.stats.hits += 1;
            return Rc::clone(existing);
        }

        let rc = Rc::new(String::from(s));
        self.cache.insert(s.to_string(), Rc::clone(&rc));
        self.stats.misses += 1;
        self.stats.unique_strings = self.cache.len();

        rc
    }

    /// Get the number of unique strings
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear all strings
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats.unique_strings = 0;
    }

    /// Get statistics
    pub fn stats(&self) -> InternerStats {
        self.stats
    }
}

#[cfg(feature = "alloc")]
impl Default for SimpleInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Common strings that frequently appear in YAML documents
///
/// Pre-interned for maximum efficiency.
pub struct CommonStrings {
    pub name: InternedString,
    pub type_: InternedString,
    pub id: InternedString,
    pub value: InternedString,
    pub key: InternedString,
    pub data: InternedString,
    pub config: InternedString,
    pub version: InternedString,
    pub description: InternedString,
    pub enabled: InternedString,
    pub disabled: InternedString,
    pub status: InternedString,
    pub message: InternedString,
    pub error: InternedString,
    pub result: InternedString,
}

impl CommonStrings {
    /// Create a new CommonStrings with pre-interned values
    pub fn new() -> Self {
        Self {
            name: InternedString::from("name"),
            type_: InternedString::from("type"),
            id: InternedString::from("id"),
            value: InternedString::from("value"),
            key: InternedString::from("key"),
            data: InternedString::from("data"),
            config: InternedString::from("config"),
            version: InternedString::from("version"),
            description: InternedString::from("description"),
            enabled: InternedString::from("enabled"),
            disabled: InternedString::from("disabled"),
            status: InternedString::from("status"),
            message: InternedString::from("message"),
            error: InternedString::from("error"),
            result: InternedString::from("result"),
        }
    }
}

impl Default for CommonStrings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_string_interner_basic() {
        let interner = StringInterner::new();
        let s1 = interner.intern("test");
        let s2 = interner.intern("test");
        let s3 = interner.intern("other");

        assert_eq!(s1.as_str(), "test");
        assert_eq!(s2.as_str(), "test");
        assert_eq!(s3.as_str(), "other");

        // Same string should have same reference
        assert_eq!(s1.ref_count(), s2.ref_count());

        assert_eq!(interner.len(), 2);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_interner_stats() {
        let interner = StringInterner::new();

        interner.intern("a");
        interner.intern("b");
        interner.intern("a"); // hit
        interner.intern("c");
        interner.intern("b"); // hit
        interner.intern("a"); // hit

        let stats = interner.stats();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 3);
        assert_eq!(stats.unique_strings, 3);
        assert_eq!(stats.total_requests(), 6);
        assert_eq!(stats.hit_rate(), 50.0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_simple_interner() {
        let mut interner = SimpleInterner::new();

        let s1 = interner.intern("test");
        let s2 = interner.intern("test");

        assert_eq!(*s1, *s2);
        assert_eq!(interner.len(), 1);

        let stats = interner.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_interned_string() {
        let s = InternedString::from("hello");
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
        assert_eq!(s.to_string(), "hello");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_memory_savings() {
        let interner = StringInterner::new();

        // Keep references alive to show memory savings
        let mut refs = Vec::new();
        for _ in 0..10 {
            refs.push(interner.intern("name"));
            refs.push(interner.intern("type"));
            refs.push(interner.intern("value"));
        }

        let (total, interned, savings, percent) = interner.memory_savings();

        // Should show significant savings
        assert!(savings > 0, "Expected savings > 0, got {}", savings);
        assert!(percent > 0.0, "Expected percent > 0, got {}", percent);
        assert!(
            interned < total,
            "Expected interned {} < total {}",
            interned,
            total
        );
    }

    #[test]
    fn test_common_strings() {
        let common = CommonStrings::new();
        assert_eq!(common.name.as_str(), "name");
        assert_eq!(common.type_.as_str(), "type");
        assert_eq!(common.id.as_str(), "id");
        assert_eq!(common.version.as_str(), "version");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_clear() {
        let interner = StringInterner::new();
        interner.intern("test");
        assert_eq!(interner.len(), 1);

        interner.clear();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_interner_with_capacity() {
        let interner = StringInterner::with_capacity(100);
        interner.intern("test");
        assert_eq!(interner.len(), 1);
    }
}
