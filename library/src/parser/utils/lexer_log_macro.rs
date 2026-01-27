//! Centralized logging macro for the lexer
//!
//! This macro wraps conditional logging for token emission and debug output.

#[macro_export]
macro_rules! lexer_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-trace")]
        {
            $crate::parser::lexer::lexer_log(format!($($arg)*));
        }
    };
}
