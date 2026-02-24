//! Anchors Debug Macro
//!
//! Provides a macro for debug logging in anchors and related anchor/alias/merge logic.
//! Enabled only when the `debug-anchors` feature is active.
//!
//! Copyright (c) 2026 YAML Library Developers

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
