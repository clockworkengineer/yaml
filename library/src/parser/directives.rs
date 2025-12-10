/// Parses YAML directives (%YAML, %TAG) at the start of a document.
/// Returns a DirectiveContext or error string.
/// YAML directive parsing (%YAML and %TAG)
///
/// Handles parsing of directives that appear at the start of YAML documents:
/// - %YAML major.minor - Specifies YAML version
/// - %TAG !handle! prefix - Defines tag shorthand
/// - %RESERVED - Reserved directives (ignored with warning)
pub fn parse_directives(
    source: &mut dyn crate::io::traits::ISource,
) -> Result<DirectiveContext, String> {
    // Inline parse_line since helpers is private
    fn parse_line(source: &mut dyn crate::io::traits::ISource) -> String {
        let mut line = String::new();
        while let Some(c) = source.current() {
            if c == '\n' || c == '\r' {
                break;
            }
            line.push(c);
            source.next();
        }
        // Skip newline
        if let Some(c) = source.current() {
            if c == '\n' || c == '\r' {
                source.next();
            }
        }
        line
    }
    let mut directives = DirectiveContext::new();
    crate::utils::skip_whitespace_and_comments(source);
    while let Some('%') = source.current() {
        let line = parse_line(source);
        let parts: Vec<_> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "%YAML" => {
                if parts.len() < 2 {
                    return Err("Missing YAML version after %YAML directive".to_string());
                }
                let version = parts[1];
                let mut split = version.split('.');
                let major = split
                    .next()
                    .and_then(|s| s.parse::<u8>().ok())
                    .ok_or("Invalid YAML major version")?;
                let minor = split
                    .next()
                    .and_then(|s| s.parse::<u8>().ok())
                    .ok_or("Invalid YAML minor version")?;
                directives.set_version(major, minor)?;
            }
            "%TAG" => {
                if parts.len() < 3 {
                    return Err("Missing handle or prefix in %TAG directive".to_string());
                }
                let handle = parts[1].to_string();
                let prefix = parts[2].to_string();
                directives.add_tag_prefix(handle, prefix);
            }
            _ => {
                // Reserved or unknown directive, skip or warn
            }
        }
        crate::utils::skip_whitespace_and_comments(source);
    }
    Ok(directives)
}

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::string::String;

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

    /// Returns true if the YAML version is 1.1
    pub fn is_yaml_11(&self) -> bool {
        matches!(self.yaml_version, Some((1, 1)))
    }
}
