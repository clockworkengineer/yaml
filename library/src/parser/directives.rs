//! YAML directive parsing (%YAML and %TAG)
//!
//! Handles parsing of directives that appear at the start of YAML documents:
//! - %YAML major.minor - Specifies YAML version
//! - %TAG !handle! prefix - Defines tag shorthand
//! - %RESERVED - Reserved directives (ignored with warning)

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::io::traits::ISource;

/// Stores directive information for a YAML document
#[derive(Clone, Debug, Default)]
pub struct DirectiveContext {
    /// YAML version (e.g., "1.2")
    pub yaml_version: Option<(u8, u8)>,

    /// Tag prefix mappings: handle -> prefix
    /// Example: "!e!" -> "tag:example.com,2000:app/"
    pub tag_prefixes: HashMap<String, String>,
}

impl DirectiveContext {
    /// Create a new directive context with default tag prefixes
    ///
    /// Per YAML 1.2 spec, two tag handles are defined by default:
    /// - `!!` resolves to `tag:yaml.org,2002:` (standard YAML types)
    /// - `!` resolves to `!` (local tags)
    pub fn new() -> Self {
        let mut tag_prefixes = HashMap::new();

        // Add default tag prefixes per YAML 1.2 specification
        tag_prefixes.insert("!!".to_string(), "tag:yaml.org,2002:".to_string());
        tag_prefixes.insert("!".to_string(), "!".to_string());

        Self {
            yaml_version: None,
            tag_prefixes,
        }
    }

    /// Set the YAML version
    pub fn set_version(&mut self, major: u8, minor: u8) -> Result<(), String> {
        // Check for duplicate YAML directive
        if self.yaml_version.is_some() {
            return Err("Duplicate YAML directive".to_string());
        }

        // Validate version (only 1.1 and 1.2 are standard)
        if major != 1 {
            return Err(alloc::format!("Invalid YAML major version: {}", major));
        }
        // Per YAML spec, parsers should accept future minor versions
        // We support 1.1 and 1.2, but don't error on higher minor versions
        // (just use 1.2 behavior)

        self.yaml_version = Some((major, minor));
        Ok(())
    }

    /// Register a tag prefix mapping
    pub fn add_tag_prefix(&mut self, handle: String, prefix: String) {
        self.tag_prefixes.insert(handle, prefix);
    }

    /// Resolve a tag handle to its full prefix
    ///
    /// Expands tag handles like `!e!mytype` to full URIs like `tag:example.com,2000:app/mytype`
    /// based on registered %TAG directives. If no handle matches, returns the tag as-is.
    ///
    /// Longer handles are matched first (e.g., `!e!` before `!`) to ensure correct resolution.
    pub fn resolve_tag(&self, tag: &str) -> String {
        // Resolve default !! handle to the YAML 1.2 prefix
        if tag.starts_with("!!") {
            let suffix = &tag[2..];
            // Use the registered default prefix for !! if present, otherwise fallback
            let default_prefix = self
                .tag_prefixes
                .get("!!")
                .cloned()
                .unwrap_or_else(|| "tag:yaml.org,2002:".to_string());
            return alloc::format!("{}{}", default_prefix, suffix);
        }
        // Find the longest matching handle
        let mut best_match: Option<(&str, &str)> = None;

        for (handle, prefix) in &self.tag_prefixes {
            if tag.starts_with(handle.as_str()) {
                // Prefer longer handles (more specific matches)
                if let Some((existing_handle, _)) = best_match {
                    if handle.len() > existing_handle.len() {
                        best_match = Some((handle, prefix));
                    }
                } else {
                    best_match = Some((handle, prefix));
                }
            }
        }

        // Apply the best match if found
        if let Some((handle, prefix)) = best_match {
            let suffix = &tag[handle.len()..];
            return alloc::format!("{}{}", prefix, suffix);
        }

        // If no handle matches, return as-is
        tag.to_string()
    }

    /// Check if this is YAML 1.1 (affects scalar parsing)
    pub fn is_yaml_11(&self) -> bool {
        matches!(self.yaml_version, Some((1, 1)))
    }

    /// Check if this is YAML 1.2 (default if no version specified)
    #[allow(dead_code)]
    pub fn is_yaml_12(&self) -> bool {
        match self.yaml_version {
            Some((1, 2)) | None => true,
            _ => false,
        }
    }
}

/// Parse all directives at the start of a document
///
/// Directives must appear before the document content and before any
/// document marker (---). Each directive is on its own line starting with %.
///
/// Returns a DirectiveContext with parsed directive information.
pub fn parse_directives(source: &mut dyn ISource) -> Result<DirectiveContext, String> {
    let mut context = DirectiveContext::new();

    // Skip any leading whitespace and comments
    crate::utils::skip_whitespace_and_comments(source);

    // Parse all directives
    while let Some('%') = source.current() {
        parse_single_directive(source, &mut context)?;
        crate::utils::skip_whitespace_and_comments(source);
    }

    Ok(context)
}

/// Parse a single directive line
fn parse_single_directive(
    source: &mut dyn ISource,
    context: &mut DirectiveContext,
) -> Result<(), String> {
    // Skip the '%' character
    source.next();

    // Read the directive name
    let directive_name = read_directive_name(source)?;

    // Skip whitespace after directive name
    skip_directive_whitespace(source);

    // Parse based on directive type
    match directive_name.as_str() {
        "YAML" => parse_yaml_directive(source, context)?,
        "TAG" => parse_tag_directive(source, context)?,
        _ => {
            // Reserved directive - ignore but skip to end of line
            skip_to_end_of_line(source);
        }
    }

    Ok(())
}

/// Read a directive name (letters only)
fn read_directive_name(source: &mut dyn ISource) -> Result<String, String> {
    let mut name = String::new();

    while let Some(c) = source.current() {
        if c.is_ascii_alphabetic() {
            name.push(c);
            source.next();
        } else {
            break;
        }
    }

    if name.is_empty() {
        return Err("Expected directive name after %".to_string());
    }

    Ok(name)
}

/// Parse %YAML major.minor directive
fn parse_yaml_directive(
    source: &mut dyn ISource,
    context: &mut DirectiveContext,
) -> Result<(), String> {
    // Read major version
    let major = read_version_number(source)?;

    // Expect '.'
    if source.current() != Some('.') {
        return Err("Expected '.' in YAML version directive".to_string());
    }
    source.next();

    // Read minor version
    let minor = read_version_number(source)?;

    // Set version in context
    context.set_version(major, minor)?;

    // After version, must have whitespace before comment or end of line
    let has_whitespace = matches!(source.current(), Some(' ') | Some('\t'));
    skip_directive_whitespace(source);

    if let Some(c) = source.current() {
        if c == '#' && !has_whitespace {
            return Err("Comment requires whitespace after YAML version".to_string());
        }
        if c != '\n' && c != '\r' && c != '#' {
            return Err(alloc::format!(
                "Invalid content after YAML version directive: '{}'",
                c
            ));
        }
    }

    // Skip to end of line (handles comment if present)
    skip_to_end_of_line(source);

    Ok(())
}

/// Parse %TAG !handle! prefix directive
fn parse_tag_directive(
    source: &mut dyn ISource,
    context: &mut DirectiveContext,
) -> Result<(), String> {
    // Read tag handle (e.g., "!e!" or "!!")
    let handle = read_tag_handle(source)?;

    // Skip whitespace
    skip_directive_whitespace(source);

    // Read tag prefix (e.g., "tag:example.com,2000:app/")
    let prefix = read_tag_prefix(source)?;

    // Register in context
    context.add_tag_prefix(handle, prefix);

    // Skip to end of line
    skip_to_end_of_line(source);

    Ok(())
}

/// Read a version number (one or two digits)
fn read_version_number(source: &mut dyn ISource) -> Result<u8, String> {
    let mut digits = String::new();

    while let Some(c) = source.current() {
        if c.is_ascii_digit() {
            digits.push(c);
            source.next();
        } else {
            break;
        }
    }

    if digits.is_empty() {
        return Err("Expected version number".to_string());
    }

    digits
        .parse::<u8>()
        .map_err(|_| "Invalid version number".to_string())
}

/// Read a tag handle (e.g., "!!", "!e!", "!my-tag!")
fn read_tag_handle(source: &mut dyn ISource) -> Result<String, String> {
    let mut handle = String::new();

    // Must start with '!'
    if source.current() != Some('!') {
        return Err("Invalid tag handle: must start with '!'".to_string());
    }
    handle.push('!');
    source.next();

    // Read characters until we hit another '!' or whitespace
    while let Some(c) = source.current() {
        if c == '!' {
            handle.push(c);
            source.next();
            break;
        } else if c.is_whitespace() || c == '\n' {
            break;
        } else if is_valid_tag_char(c) {
            handle.push(c);
            source.next();
        } else {
            return Err(alloc::format!("Invalid character in tag handle: {}", c));
        }
    }

    Ok(handle)
}

/// Read a tag prefix (URI-like string)
fn read_tag_prefix(source: &mut dyn ISource) -> Result<String, String> {
    let mut prefix = String::new();

    while let Some(c) = source.current() {
        if c == '\n' || c == '\r' || c == '#' {
            break;
        } else if c.is_whitespace() {
            // Stop at whitespace but don't consume it yet (might be followed by comment)
            break;
        } else {
            prefix.push(c);
            source.next();
        }
    }

    if prefix.is_empty() {
        return Err("Invalid TAG directive: expected tag prefix".to_string());
    }

    Ok(prefix)
}

/// Check if a character is valid in a tag handle
fn is_valid_tag_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// Skip whitespace but not newlines (for directives)
fn skip_directive_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c == ' ' || c == '\t' {
            source.next();
        } else {
            break;
        }
    }
}

/// Skip to end of line (including comments)
fn skip_to_end_of_line(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c == '\n' || c == '\r' {
            source.next();
            break;
        }
        source.next();
    }
}

/// Skip whitespace including newlines
#[allow(dead_code)]
fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c.is_whitespace() {
            source.next();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_parse_yaml_directive() {
        let mut source = Buffer::new(b"%YAML 1.2\n---\ntest");
        let context = parse_directives(&mut source).unwrap();

        assert_eq!(context.yaml_version, Some((1, 2)));
    }

    #[test]
    fn test_parse_yaml_directive_11() {
        let mut source = Buffer::new(b"%YAML 1.1\n---\ntest");
        let context = parse_directives(&mut source).unwrap();

        assert_eq!(context.yaml_version, Some((1, 1)));
    }

    #[test]
    fn test_parse_tag_directive() {
        let mut source = Buffer::new(b"%TAG !e! tag:example.com,2000:app/\n---\ntest");
        let context = parse_directives(&mut source).unwrap();

        assert_eq!(
            context.tag_prefixes.get("!e!"),
            Some(&"tag:example.com,2000:app/".to_string())
        );
    }

    #[test]
    fn test_parse_multiple_directives() {
        let mut source = Buffer::new(b"%YAML 1.2\n%TAG !e! tag:example.com,2000:app/\n---\ntest");
        let context = parse_directives(&mut source).unwrap();

        assert_eq!(context.yaml_version, Some((1, 2)));
        assert_eq!(
            context.tag_prefixes.get("!e!"),
            Some(&"tag:example.com,2000:app/".to_string())
        );
    }

    #[test]
    fn test_parse_tag_directive_primary() {
        let mut source = Buffer::new(b"%TAG !! tag:example.com,2000:app/\n");
        let context = parse_directives(&mut source).unwrap();

        assert_eq!(
            context.tag_prefixes.get("!!"),
            Some(&"tag:example.com,2000:app/".to_string())
        );
    }

    #[test]
    fn test_parse_reserved_directive() {
        // Reserved directives should be ignored without error
        let mut source = Buffer::new(b"%FOO bar baz\n---\ntest");
        let context = parse_directives(&mut source).unwrap();

        // Should have no version but should have default tag prefixes
        assert_eq!(context.yaml_version, None);
        assert_eq!(context.tag_prefixes.len(), 2); // !! and ! defaults
        assert_eq!(
            context.tag_prefixes.get("!!"),
            Some(&"tag:yaml.org,2002:".to_string())
        );
        assert_eq!(context.tag_prefixes.get("!"), Some(&"!".to_string()));
    }

    #[test]
    fn test_resolve_tag_with_prefix() {
        let mut context = DirectiveContext::new();
        context.add_tag_prefix("!e!".to_string(), "tag:example.com,2000:app/".to_string());

        let resolved = context.resolve_tag("!e!mytype");
        assert_eq!(resolved, "tag:example.com,2000:app/mytype");
    }

    #[test]
    fn test_resolve_tag_without_prefix() {
        let context = DirectiveContext::new();

        let resolved = context.resolve_tag("!mytag");
        assert_eq!(resolved, "!mytag");
    }

    #[test]
    fn test_resolve_default_tag_prefix() {
        let context = DirectiveContext::new();

        // Test default !! prefix resolution
        let resolved = context.resolve_tag("!!str");
        assert_eq!(resolved, "tag:yaml.org,2002:str");

        let resolved = context.resolve_tag("!!int");
        assert_eq!(resolved, "tag:yaml.org,2002:int");

        let resolved = context.resolve_tag("!!null");
        assert_eq!(resolved, "tag:yaml.org,2002:null");
    }

    #[test]
    fn test_resolve_local_tag() {
        let context = DirectiveContext::new();

        // Local tags (single !) should stay as-is
        let resolved = context.resolve_tag("!custom");
        assert_eq!(resolved, "!custom");
    }

    #[test]
    fn test_invalid_yaml_version() {
        let mut source = Buffer::new(b"%YAML 2.0\n");
        let result = parse_directives(&mut source);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid YAML major version"));
    }

    #[test]
    fn test_yaml_directive_missing_dot() {
        let mut source = Buffer::new(b"%YAML 1 2\n");
        let result = parse_directives(&mut source);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected '.'"));
    }

    #[test]
    fn test_tag_directive_invalid_handle() {
        let mut source = Buffer::new(b"%TAG invalid tag:example.com\n");
        let result = parse_directives(&mut source);

        assert!(result.is_err());
    }
}
