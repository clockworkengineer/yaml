//! Module: parser/document/loop_guards.rs
//!
//! Provides macros and utilities for safe loop iteration in parsing functions.
//! These guards prevent infinite loops and resource exhaustion attacks.

/// Default maximum number of loop iterations for parsing loops.
/// This prevents infinite loops in case of malformed input or parser bugs.
pub const MAX_LOOP_ITERATIONS: usize = 100_000;

/// Default maximum number of items in a sequence.
/// This prevents memory exhaustion from extremely large sequences.
pub const MAX_SEQUENCE_ITEMS: usize = 50_000;

/// Default maximum number of pairs in a mapping.
/// This prevents memory exhaustion from extremely large mappings.
pub const MAX_MAPPING_PAIRS: usize = 50_000;

/// Macro to create and check loop iteration guard.
///
/// This macro generates the boilerplate code for preventing infinite loops
/// in parsing functions. It declares a counter variable and checks it on each
/// iteration.
///
/// # Usage
///
/// ```ignore
/// loop_guard_init!(counter);
/// while let Some(c) = source.current() {
///     loop_guard_check!(counter, MAX_LOOP_ITERATIONS, "Sequence parsing")?;
///     // ... parsing logic
/// }
/// ```
#[macro_export]
macro_rules! loop_guard_init {
    ($counter:ident) => {
        let mut $counter: usize = 0;
    };
}

/// Macro to check loop iteration count and return error if exceeded.
///
/// # Arguments
///
/// * `$counter` - The counter variable name (from loop_guard_init!)
/// * `$max` - Maximum allowed iterations
/// * `$context` - Description of what's being parsed (for error message)
///
/// # Returns
///
/// Returns `Err(String)` if limit exceeded, continues execution otherwise.
#[macro_export]
macro_rules! loop_guard_check {
    ($counter:ident, $max:expr, $context:expr) => {{
        $counter += 1;
        if $counter >= $max {
            return Err(crate::parser::document::error_builder::limit_error(
                $context, $max, "loop iterations"
            ));
        }
    }};
}

/// Macro to check collection size and return error if exceeded.
///
/// # Arguments
///
/// * `$collection` - The Vec or other collection to check
/// * `$max` - Maximum allowed size
/// * `$type_name` - Name of the collection type (for error message)
///
/// # Returns
///
/// Returns `Err(String)` if limit exceeded, continues execution otherwise.
#[macro_export]
macro_rules! collection_size_check {
    ($collection:expr, $max:expr, $type_name:expr) => {
        if $collection.len() >= $max {
            return Err(crate::parser::document::error_builder::limit_error(
                $type_name, $max, "items"
            ));
        }
    };
}

/// Combined guard for loops that build collections.
///
/// This macro combines both iteration counting and collection size checking
/// into a single convenient check.
///
/// # Usage
///
/// ```ignore
/// loop_guard_init!(counter);
/// let mut items = Vec::new();
/// 
/// while let Some(c) = source.current() {
///     combined_loop_guard!(counter, items, MAX_LOOP_ITERATIONS, MAX_ITEMS, "Sequence")?;
///     // ... parsing logic that adds to items
/// }
/// ```
#[macro_export]
macro_rules! combined_loop_guard {
    ($counter:ident, $collection:expr, $max_iter:expr, $max_size:expr, $context:expr) => {{
        $counter += 1;
        if $counter >= $max_iter {
            return Err(crate::parser::document::error_builder::limit_error(
                &format!("{} parsing", $context), $max_iter, "loop iterations"
            ));
        }
        if $collection.len() >= $max_size {
            return Err(crate::parser::document::error_builder::limit_error(
                $context, $max_size, "items"
            ));
        }
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_loop_guard_allows_normal_iterations() {
        fn test_function() -> Result<(), String> {
            loop_guard_init!(counter);
            let mut items = Vec::new();
            
            for i in 0..100 {
                loop_guard_check!(counter, 1000, "Test");
                items.push(i);
            }
            
            Ok(())
        }
        
        assert!(test_function().is_ok());
    }

    #[test]
    fn test_loop_guard_catches_infinite_loop() {
        fn test_function() -> Result<(), String> {
            loop_guard_init!(counter);
            
            loop {
                loop_guard_check!(counter, 100, "Test");
                // Simulated infinite loop
                if counter > 200 {
                    break; // Safety for test
                }
            }
            
            Ok(())
        }
        
        let result = test_function();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeded maximum loop iterations"));
    }

    #[test]
    fn test_collection_size_check_normal() {
        fn test_function() -> Result<(), String> {
            let items = vec![1, 2, 3, 4, 5];
            collection_size_check!(items, 100, "Test collection");
            Ok(())
        }
        
        assert!(test_function().is_ok());
    }

    #[test]
    fn test_collection_size_check_exceeds_limit() {
        fn test_function() -> Result<(), String> {
            let items = vec![1; 150];
            collection_size_check!(items, 100, "Test collection");
            Ok(())
        }
        
        let result = test_function();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("exceeded") || err.contains("items"), "Error: {}", err);
    }

    #[test]
    fn test_combined_guard_normal() {
        fn test_function() -> Result<(), String> {
            loop_guard_init!(counter);
            let mut items = Vec::new();
            
            for i in 0..50 {
                combined_loop_guard!(counter, items, 1000, 100, "Test");
                items.push(i);
            }
            
            Ok(())
        }
        
        assert!(test_function().is_ok());
    }

    #[test]
    fn test_combined_guard_catches_iteration_limit() {
        fn test_function() -> Result<(), String> {
            loop_guard_init!(counter);
            let mut items = Vec::new();
            
            for i in 0..150 {
                combined_loop_guard!(counter, items, 100, 1000, "Test");
                items.push(i);
            }
            
            Ok(())
        }
        
        let result = test_function();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeded maximum loop iterations"));
    }

    #[test]
    fn test_combined_guard_catches_size_limit() {
        fn test_function() -> Result<(), String> {
            loop_guard_init!(counter);
            let mut items = Vec::new();
            
            for i in 0..150 {
                combined_loop_guard!(counter, items, 1000, 100, "Test");
                items.push(i);
            }
            
            Ok(())
        }
        
        let result = test_function();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("exceeded") || err.contains("items"), "Error: {}", err);
    }
}
