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
