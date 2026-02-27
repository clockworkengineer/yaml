//! String Escaping Utilities for YAML Library
//!
//! This module provides shared escaping logic for JSON, XML, and YAML stringification.
//! It ensures consistent and correct escaping of special characters across different formats.
//!
//! # Features
//! - Escape special characters for JSON, XML, and YAML
//! - Centralized, reusable implementation
//!
//! # Usage
//! Use these functions in format-specific modules to ensure proper escaping of output strings.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_for_json_basic() {
        assert_eq!(escape_for_json("foo"), "foo");
        assert_eq!(escape_for_json("\"\\\n\r\t"), "\\\"\\\\\\n\\r\\t");
        assert_eq!(escape_for_json("abc\u{1F}"), "abc\\u001f");
    }

    #[test]
    fn test_escape_for_json_control_chars() {
        let input = "\x01\x02\x03";
        let expected = "\\u0001\\u0002\\u0003";
        assert_eq!(escape_for_json(input), expected);
    }

    #[test]
    fn test_escape_for_xml_basic() {
        assert_eq!(escape_for_xml("foo"), "foo");
        assert_eq!(escape_for_xml("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn test_escape_for_xml_mixed() {
        let input = "a <b> & 'c' \"d\"";
        let expected = "a &lt;b&gt; &amp; &apos;c&apos; &quot;d&quot;";
        assert_eq!(escape_for_xml(input), expected);
    }
}
