/// bencode
pub mod bencode;
/// default
pub mod default;
/// json
pub mod json;
/// toml
pub mod toml;
/// xml
pub mod xml;
/// Formatting options and control
#[cfg(feature = "alloc")]
pub mod format;
/// Custom serializer support
#[cfg(feature = "alloc")]
pub mod serializer;
/// Streaming serialization
#[cfg(feature = "alloc")]
pub mod streaming;
