/// bencode
pub mod bencode;
/// default
pub mod default;
/// Formatting options and control
#[cfg(feature = "alloc")]
pub mod format;
/// json
pub mod json;
/// Custom serializer support
#[cfg(feature = "alloc")]
pub mod serializer;
/// Streaming serialization
#[cfg(feature = "alloc")]
pub mod streaming;
/// toml
pub mod toml;
/// xml
pub mod xml;
