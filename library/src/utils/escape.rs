//! Shared string escaping utilities for stringify modules
//!
//! This module centralizes escaping logic for JSON, XML, and YAML
//! representations so that format-specific modules can reuse a
//! consistent implementation.

#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::string::String;

/// Escapes special characters in a string for JSON representation.
///
/// Mirrors the behavior previously implemented in stringify/json.rs.
pub fn escape_for_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use alloc::format;
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

/// Escapes special characters in a string for XML text content.
///
/// Mirrors the behavior previously implemented in stringify/xml.rs.
pub fn escape_for_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// Escapes special characters in a string for double-quoted YAML representation.
///
/// This is a thin wrapper around the existing `escape_double` logic in
/// stringify/default.rs to make it available to other modules without
/// duplicating behavior. For now, callers in YAML formatter continue to
/// use their local helper; this API exists for potential future reuse.
pub fn escape_yaml_double<F>(s: &str, escape_impl: F) -> String
where
    F: Fn(&str) -> String,
{
    escape_impl(s)
}

/// Escapes single quotes in a string for single-quoted YAML representation.
pub fn escape_yaml_single<F>(s: &str, escape_impl: F) -> String
where
    F: Fn(&str) -> String,
{
    escape_impl(s)
}
