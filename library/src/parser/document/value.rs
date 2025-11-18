//! Module: parser/document/value.rs

use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::document::helpers::{parse_error, parse_quoted_scalar, skip_whitespace};
use crate::parser::document::inline::{parse_inline_mapping, parse_inline_sequence};
use crate::parser::document::scalar::parse_scalar;
use crate::utils::*;

/// Validates if a string is valid base64 format
fn is_base64(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    // Check length (must be multiple of 4 for proper base64)
    if s.len() % 4 != 0 {
        return false;
    }

    // Check characters (only A-Z, a-z, 0-9, +, /, and = for padding)
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '+' | '/' => continue,
            '=' => continue, // Padding character
            _ => return false,
        }
    }

    // Check padding rules
    let padding_count = s.chars().rev().take_while(|&c| c == '=').count();
    if padding_count > 2 {
        return false;
    }

    true
}
///
/// Handles type coercion for various YAML tags including string (!str),
/// integer (!int), float (!float), boolean (!bool), and timestamp (!timestamp).
/// Returns None if the coercion is not possible or the tag is unsupported.
///
/// # Arguments
///
/// * `tag` - The YAML tag string (e.g., "!!str", "!!int")
/// * `node` - The Node to coerce
///
/// # Returns
///
/// Some(Node) with the coerced type, or None if coercion failed
fn try_coerce_tag(tag: &str, node: Node) -> Option<Node> {
    match tag {
        "!!str" | "!str" => {
            let s = match node {
                Node::Str(s, _, _) => s,
                Node::Number(Numeric::Integer(i)) => i.to_string(),
                Node::Number(Numeric::Float(f)) => f.to_string(),
                Node::Boolean(b) => b.to_string(),
                Node::None => String::new(),
                _ => return None,
            };

            return Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None));
        }
        "!!int" | "!int" => match node {
            Node::Number(Numeric::Integer(i)) => {
                return Some(Node::Number(Numeric::Integer(i)));
            }
            Node::Number(Numeric::Float(f)) => {
                if (f.trunc() - f).abs() < std::f64::EPSILON {
                    return Some(Node::Number(Numeric::Integer(f as i64)));
                }
                return None;
            }
            Node::Str(s, _, _) => {
                if let Ok(i) = s.parse::<i64>() {
                    return Some(Node::Number(Numeric::Integer(i)));
                }
                return None;
            }
            _ => return None,
        },
        "!!float" | "!float" => match node {
            Node::Number(Numeric::Float(f)) => return Some(Node::Number(Numeric::Float(f))),
            Node::Number(Numeric::Integer(i)) => {
                return Some(Node::Number(Numeric::Float(i as f64)));
            }
            Node::Str(s, _, _) => {
                if let Ok(f) = s.parse::<f64>() {
                    return Some(Node::Number(Numeric::Float(f)));
                }
                return None;
            }
            _ => return None,
        },
        "!!bool" | "!bool" => match node {
            Node::Boolean(b) => return Some(Node::Boolean(b)),
            Node::Str(s, _, _) => {
                let sl = s.to_ascii_lowercase();
                if sl == "true" {
                    return Some(Node::Boolean(true));
                }
                if sl == "false" {
                    return Some(Node::Boolean(false));
                }
                return None;
            }
            _ => return None,
        },
        "!!null" | "!null" => {
            return Some(Node::None);
        }
        "!!timestamp" | "!timestamp" => match node {
            Node::Str(s, _, _) => {
                return Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None));
            }
            Node::Number(Numeric::Integer(i)) => {
                return Some(Node::Str(
                    i.to_string(),
                    QuoteType::Unquoted,
                    BlockStyle::None,
                ));
            }
            Node::Number(Numeric::Float(f)) => {
                return Some(Node::Str(
                    f.to_string(),
                    QuoteType::Unquoted,
                    BlockStyle::None,
                ));
            }
            _ => return None,
        },
        "!!set" | "!set" => match node {
            // Convert mapping with null values to a set
            Node::Mapping(pairs) => {
                let mut set_items = Vec::new();
                for (key, value) in pairs {
                    match value {
                        Node::None => {
                            // Only add keys where value is null (which is what sets are)
                            set_items.push(key);
                        }
                        _ => {
                            // If any value is not null, it's not a valid set mapping
                            return None;
                        }
                    }
                }
                return Some(Node::Set(set_items));
            }
            // Convert array to a set (remove duplicates)
            Node::Array(items) => {
                let mut unique_items = Vec::new();
                for item in items {
                    if !unique_items.contains(&item) {
                        unique_items.push(item);
                    }
                }
                return Some(Node::Set(unique_items));
            }
            // Single value becomes a set with one element
            _ => {
                return Some(Node::Set(vec![node]));
            }
        },
        "!!binary" | "!binary" => match node {
            Node::Str(s, _, _) => {
                // Validate base64 format and decode if valid
                let clean_input = s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
                if is_base64(&clean_input) {
                    // Store as a tagged string node to preserve the binary nature
                    return Some(Node::Tagged(
                        Box::new(Node::Str(
                            clean_input,
                            QuoteType::Unquoted,
                            BlockStyle::None,
                        )),
                        "!!binary".to_string(),
                    ));
                }
                return None;
            }
            _ => return None,
        },
        "!!omap" | "!omap" => match node {
            // Ordered mapping - convert to array of single-key mappings
            Node::Array(items) => {
                let mut omap_items = Vec::new();
                for item in items {
                    match &item {
                        Node::Mapping(pairs) if pairs.len() == 1 => {
                            omap_items.push(item);
                        }
                        _ => return None, // Invalid omap format
                    }
                }
                return Some(Node::Tagged(
                    Box::new(Node::Array(omap_items)),
                    "!!omap".to_string(),
                ));
            }
            Node::Mapping(pairs) => {
                // Convert mapping to array of single-key mappings
                let mut omap_items = Vec::new();
                for (key, value) in pairs {
                    omap_items.push(Node::Mapping(vec![(key, value)]));
                }
                return Some(Node::Tagged(
                    Box::new(Node::Array(omap_items)),
                    "!!omap".to_string(),
                ));
            }
            _ => return None,
        },
        "!!pairs" | "!pairs" => match node {
            // Pairs - array of key-value pairs
            Node::Array(items) => {
                let mut pairs_items = Vec::new();
                for item in items {
                    match &item {
                        Node::Mapping(pairs) if pairs.len() == 1 => {
                            pairs_items.push(item);
                        }
                        Node::Array(arr) if arr.len() == 2 => {
                            // Convert [key, value] to {key: value}
                            let key = arr[0].clone();
                            let value = arr[1].clone();
                            pairs_items.push(Node::Mapping(vec![(key, value)]));
                        }
                        _ => return None, // Invalid pairs format
                    }
                }
                return Some(Node::Tagged(
                    Box::new(Node::Array(pairs_items)),
                    "!!pairs".to_string(),
                ));
            }
            _ => return None,
        },
        // Support for YAML 1.1 compatibility tags
        "!!yaml" | "!yaml" => match node {
            // YAML version tag - just preserve as string
            Node::Str(s, qt, bs) => Some(Node::Tagged(
                Box::new(Node::Str(s, qt, bs)),
                "!!yaml".to_string(),
            )),
            _ => None,
        },
        // Support for hexadecimal integers
        "!!int:hex" | "!int:hex" => match node {
            Node::Str(s, _, _) => {
                let clean = s.trim_start_matches("0x").trim_start_matches("0X");
                if let Ok(i) = i64::from_str_radix(clean, 16) {
                    Some(Node::Number(Numeric::Integer(i)))
                } else {
                    None
                }
            }
            _ => None,
        },
        // Support for octal integers
        "!!int:oct" | "!int:oct" => match node {
            Node::Str(s, _, _) => {
                let clean = s.trim_start_matches("0o").trim_start_matches("0");
                if let Ok(i) = i64::from_str_radix(clean, 8) {
                    Some(Node::Number(Numeric::Integer(i)))
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => return None,
    }
}

/// Parses a YAML value, handling tags, anchors, aliases, and various value types.
///
/// Processes YAML values including tagged values (!tag), anchored values (&anchor),
/// aliases (*alias), inline collections, quoted scalars, and plain scalars.
/// Handles type coercion based on tags and resolves anchors and aliases.
///
/// Tag handles are resolved using the directive context.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing the parsed Node value or an error string
pub(crate) fn parse_value(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    if source.current() == Some('!') {
        source.next();

        let rest = collect_until(source, |c| c == CHAR_SPACE || c == CHAR_NEWLINE);
        let tag = format!("!{}", rest);
        if tag.trim().is_empty() {
            return Err(parse_error(source, "Empty tag"));
        }

        // Resolve tag handle using directive context
        let resolved_tag = directives.resolve_tag(&tag);

        skip_whitespace(source);

        let inner = match source.current() {
            Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim(), directives)
            }
            Some('-') => {
                let st = source.save_state();
                source.next();
                let next_ch = source.current();
                source.restore_state(st);

                if let Some(ch) = next_ch {
                    if source.is_whitespace(ch) || ch == CHAR_NEWLINE {
                        let nested_indent = source.get_current_indent_level();
                        crate::parser::document::parse_sequence(source, nested_indent, directives)?
                    } else {
                        parse_value(source, directives)?
                    }
                } else {
                    return Err(parse_error(source, ERR_UNEXPECTED_EOF_AFTER_ANCHOR));
                }
            }
            Some('\n') => {
                // Tag followed by newline - content is on following lines
                source.next(); // consume the newline
                skip_whitespace(source);

                // Check if there's indented content
                let current_indent = source.get_current_indent_level();
                if current_indent > 0 {
                    // Parse as block structure
                    crate::parser::document::parse_document_contents(
                        source,
                        current_indent,
                        directives,
                    )?
                } else {
                    // No indented content, treat as null
                    Node::None
                }
            }
            Some(_) => parse_value(source, directives)?,
            None => return Err(parse_error(source, ERR_UNEXPECTED_EOF_AFTER_ANCHOR)),
        };

        // Use resolved tag for coercion and storage
        if let Some(coerced) = try_coerce_tag(&resolved_tag, inner.clone()) {
            return Ok(coerced);
        }

        return Ok(Node::Tagged(Box::new(inner), resolved_tag));
    }

    if source.current() == Some(CHAR_ASTERISK) {
        source.next();
        let name = collect_until(source, |c| {
            // Alias names can contain any character except:
            // - Whitespace (space, tab, newline, carriage return)
            // - Flow indicators: [ ] { } ,
            // - Comment indicator: #
            c == CHAR_NEWLINE
                || c == CHAR_HASH
                || c.is_whitespace()
                || c == CHAR_COMMA
                || c == CHAR_LBRACKET
                || c == CHAR_RBRACKET
                || c == CHAR_LBRACE
                || c == CHAR_RBRACE
        });
        if name.trim().is_empty() {
            return Err(parse_error(source, ERR_EMPTY_ALIAS_NAME));
        }
        return Ok(Node::Alias(name));
    }

    if source.current() == Some(CHAR_AMPERSAND) {
        source.next();
        let name = collect_until(source, |c| {
            // Anchor names can contain any character except:
            // - Whitespace (space, tab, newline, carriage return)
            // - Flow indicators: [ ] { } ,
            // - Comment indicator: #
            c == CHAR_SPACE
                || c == CHAR_TAB
                || c == CHAR_NEWLINE
                || c == CHAR_CARRIAGE_RETURN
                || c == CHAR_HASH
                || c == CHAR_COMMA
                || c == CHAR_LBRACKET
                || c == CHAR_RBRACKET
                || c == CHAR_LBRACE
                || c == CHAR_RBRACE
        });
        if name.trim().is_empty() {
            return Err(parse_error(source, ERR_EMPTY_ANCHOR_NAME));
        }
        skip_whitespace(source);
        // Handle both Unix (\n) and Windows (\r\n) line endings
        if source.current() == Some(CHAR_CARRIAGE_RETURN) {
            source.next();
        }
        if source.current() == Some(CHAR_NEWLINE) {
            source.next();
            skip_whitespace(source);
            let nested_indent = source.get_current_indent_level();
            let node = crate::parser::document::parse_document_contents(
                source,
                nested_indent,
                directives,
            )?;
            // Check for nested anchors (not allowed)
            if matches!(node, Node::Anchored(_, _)) {
                return Err(parse_error(source, "A node cannot have multiple anchors"));
            }
            return Ok(Node::Anchored(Box::new(node), name));
        }
        let node = match source.current() {
            Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim(), directives)
            }
            Some(_) => parse_value(source, directives)?,
            None => return Err(parse_error(source, ERR_UNEXPECTED_EOF_AFTER_ANCHOR)),
        };
        // Check for nested anchors (not allowed)
        if matches!(node, Node::Anchored(_, _)) {
            return Err(parse_error(source, "A node cannot have multiple anchors"));
        }
        return Ok(Node::Anchored(Box::new(node), name));
    }

    match source.current() {
        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives),
        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives),
        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
            let raw = parse_quoted_scalar(source)?;
            let trimmed = raw.trim();
            if trimmed.starts_with(CHAR_DOUBLE_QUOTE)
                && trimmed.ends_with(CHAR_DOUBLE_QUOTE)
                && trimmed.contains('\n')
            {
                let parsed = parse_scalar(trimmed, directives);
                return if let Node::Str(content, QuoteType::Double, _style) = parsed {
                    let had_indent_after_newline = trimmed.contains("\n ");
                    let had_explicit_escaped_nl_at_end = trimmed.ends_with("\\\n\"");
                    if had_indent_after_newline || had_explicit_escaped_nl_at_end {
                        Ok(Node::Str(content, QuoteType::Unquoted, BlockStyle::Literal))
                    } else {
                        Ok(Node::Str(content, QuoteType::Double, BlockStyle::None))
                    }
                } else {
                    Ok(parsed)
                };
            }
            Ok(parse_scalar(trimmed, directives))
        }
        Some(_) => {
            let value = collect_until(source, |c| c == CHAR_NEWLINE || c == CHAR_HASH);
            let trimmed = value.trim();
            
            // Get the indent level of the LINE where the value started (not character position)
            // We need to look back to find the line's actual indentation
            // For now, use a simple heuristic: count leading spaces in the collected value
            // If value starts without leading space, line indent was 0 or we're inline
            let value_line_indent = value.len() - value.trim_start().len();

            if trimmed.starts_with(STR_LITERAL_BLOCK) || trimmed.starts_with(STR_FOLDED_BLOCK) {
                let first_ch = trimmed.chars().next().unwrap();
                let rest = trimmed[1..].trim();

                // Validate block scalar modifiers
                // Format: |[1-9]?[+-]? or >[1-9]?[+-]?
                let mut chars = rest.chars();
                if let Some(first) = chars.next() {
                    if first.is_ascii_digit() {
                        // Check if indent indicator is 0 (invalid)
                        if first == '0' {
                            return Err(parse_error(
                                source,
                                "Block scalar indentation indicator must be between 1-9, not 0",
                            ));
                        }
                        // Check if there's a second digit (invalid - must be single digit 1-9)
                        if let Some(second) = chars.clone().next() {
                            if second.is_ascii_digit() {
                                return Err(parse_error(
                                    source,
                                    "Block scalar indentation indicator must be a single digit 1-9",
                                ));
                            }
                        }
                    }
                }

                let valid_header_rest = rest
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '+' || c == '-');
                if !valid_header_rest {
                    if !trimmed.is_empty() {
                        return Ok(parse_scalar(trimmed, directives));
                    } else {
                        return Ok(Node::None);
                    }
                }
                let is_folded = first_ch == STR_FOLDED_BLOCK.chars().next().unwrap();

                let keep_trailing = trimmed.contains('+');
                if source.current() == Some(CHAR_NEWLINE) {
                    source.next();
                }

                let mut raw_lines: Vec<String> = Vec::new();
                let mut first_indent: Option<usize> = None;
                loop {
                    if source.current().is_none() {
                        break;
                    }
                    let st_line = source.save_state();
                    let mut cur_indent = 0usize;
                    while let Some(CHAR_SPACE) = source.current() {
                        cur_indent += 1;
                        source.next();
                    }
                    let cur_is_newline = source.current() == Some(CHAR_NEWLINE);
                    source.restore_state(st_line);

                    if first_indent.is_none() {
                        if cur_is_newline {
                            let _ = collect_until(source, |c| c == CHAR_NEWLINE);
                            if source.current() == Some(CHAR_NEWLINE) {
                                source.next();
                            }
                            raw_lines.push(String::new());
                            continue;
                        } else {
                            first_indent = Some(cur_indent);
                        }
                    } else if !cur_is_newline && cur_indent < first_indent.unwrap() {
                        break;
                    }

                    let raw_line = collect_until(source, |c| c == CHAR_NEWLINE);
                    if source.current() == Some(CHAR_NEWLINE) {
                        source.next();
                    }
                    raw_lines.push(raw_line);
                }

                if !keep_trailing {
                    while matches!(raw_lines.last(), Some(s) if s.is_empty()) {
                        raw_lines.pop();
                    }
                }

                let fi = first_indent.unwrap_or(0);
                let mut norm_lines: Vec<String> = Vec::with_capacity(raw_lines.len());
                // Strip base indentation from all lines (both literal and folded)
                for l in raw_lines.iter() {
                    if l.is_empty() {
                        norm_lines.push(String::new());
                    } else {
                        let lead = l.chars().take_while(|&ch| ch == CHAR_SPACE).count();
                        let strip = fi.min(lead);
                        let stripped: String = l.chars().skip(strip).collect();
                        norm_lines.push(stripped);
                    }
                }

                let out = if is_folded {
                    let mut out = String::new();
                    let mut i = 0usize;
                    while i < norm_lines.len() {
                        let line = &norm_lines[i];
                        if line.is_empty() {
                            out.push(CHAR_NEWLINE);
                            i += 1;
                            continue;
                        }
                        let is_indented = line.starts_with(' ');
                        if !is_indented {
                            let mut j = i;
                            let mut first = true;
                            while j < norm_lines.len() {
                                let l = &norm_lines[j];
                                if l.is_empty() || l.starts_with(' ') {
                                    break;
                                }
                                if !first {
                                    out.push(CHAR_SPACE);
                                }
                                out.push_str(l);
                                first = false;
                                j += 1;
                            }
                            out.push(CHAR_NEWLINE);
                            i = j;
                        } else {
                            out.push_str(line);
                            out.push(CHAR_NEWLINE);
                            i += 1;
                        }
                    }
                    if out.ends_with('\n') {
                        out.pop();
                    }
                    out
                } else {
                    let mut out = String::new();
                    for (idx, l) in norm_lines.iter().enumerate() {
                        out.push_str(l);
                        if idx + 1 < norm_lines.len() {
                            out.push(CHAR_NEWLINE);
                        }
                    }
                    out
                };

                return Ok(Node::Str(
                    out,
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::Literal,
                ));
            }
            // Handle plain scalar (potentially multiline)
            if !trimmed.is_empty() {
                // Check if this plain scalar continues on subsequent lines
                let mut parts = vec![value.clone()];
                let mut has_empty_line = false;
                
                // Collect continuation lines
                loop {
                    // Check if we're at a newline
                    if source.current() != Some(CHAR_NEWLINE) {
                        break;
                    }
                    
                    // Save state to potentially restore
                    let state = source.save_state();
                    source.next(); // Skip newline
                    
                    // Check indent of next line
                    let next_indent = source.get_current_indent_level();
                    
                    // Empty line - this might end the scalar or be part of it
                    if source.current() == Some(CHAR_NEWLINE) {
                        has_empty_line = true;
                        source.restore_state(state);
                        break;
                    }
                    
                    // EOF ends the scalar
                    if source.current().is_none() {
                        source.restore_state(state);
                        break;
                    }
                    
                    // Line with indent 0 ends the scalar
                    // (Continuation lines must be indented)
                    if next_indent == 0 {
                        source.restore_state(state);
                        break;
                    }
                    
                    // Check for special indicators that would end the scalar
                    if let Some(c) = source.current() {
                        // Dash at start of line (sequence item), colon, or other indicators
                        if matches!(c, '-' | '?' | ':' | '#' | '&' | '*' | '!' | '|' | '>' | '\'' | '"' | '%' | '@' | '`') {
                            source.restore_state(state);
                            break;
                        }
                    }
                    
                    // This is a continuation line - collect it (DON'T restore state)
                    let cont_line = collect_until(source, |c| c == CHAR_NEWLINE || c == CHAR_HASH);
                    if !cont_line.trim().is_empty() {
                        parts.push(cont_line);
                    }
                    // Now we're at the newline after the continuation line, loop will check again
                }
                
                // Join lines with space folding
                if parts.len() == 1 && !has_empty_line {
                    Ok(parse_scalar(trimmed, directives))
                } else {
                    // Multiline plain scalar - fold lines together
                    let combined = parts.join(" ");
                    Ok(parse_scalar(combined.trim(), directives))
                }
            } else {
                Ok(Node::None)
            }
        }
        None => Ok(Node::None),
    }
}
