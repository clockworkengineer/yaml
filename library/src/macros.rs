//! Lexer Debug Logging Macro for YAML Library
//!
//! This module defines a macro for centralized, conditional debug logging in the lexer.
//! The macro is enabled only when the `debug-trace` feature is active, and routes output
//! through the lexer logging facility. Use this macro for token emission and lexer debug output.
//!
//! # Features
//! - Conditional debug logging for the lexer
//! - Controlled by the `debug-trace` feature
//! - Integrates with the lexer logging system
//!
//! # Usage
//! Use the `lexer_debug!` macro for debug output in lexer code paths.

#[macro_export]
macro_rules! lexer_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-trace")]
        {
            $crate::parser::lexer::lexer_log(format!($($arg)*));
        }
    };
}
