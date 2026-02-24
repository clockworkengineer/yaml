//! Debug Logging Macro for YAML Anchors
//!
//! This module defines a macro for debug logging in anchor, alias, and merge logic.
//! The macro is enabled only when the `debug-anchors` feature is active, and uses the `log` crate
//! for output. It is used throughout anchor-related code for conditional debug output.
//!
//! # Features
//! - Conditional debug logging for anchor logic
//! - Controlled by the `debug-anchors` feature
//! - Integrates with the `log` crate
//!
//! # Usage
//! Use the `anchors_debug!` macro for debug output in anchor/alias/merge code paths.
/// Macro for debug logging in anchors.rs and related anchor/alias/merge logic
#[macro_export]
macro_rules! anchors_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-anchors")]
        {
            log::debug!($($arg)*);
        }
    };
}
