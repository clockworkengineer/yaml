//! Document-level YAML parser split into several focused source files.
//! This module re-exports the primary parsing entry points used by the rest
//! of the crate and contains unit tests ported from the former single-file
//! implementation.

mod anchors;
mod block_scalar;
mod bridge;
mod context;
mod error_builder;
mod explicit_key;
mod helpers;
mod inline;
mod inline_tokens;
mod loop_guards;
mod mapping;
mod mapping_tokens;
mod scalar;
mod sequence;
mod sequence_tokens;
mod tokens;
mod value;
mod value_tokens;
// token modules now grouped under tokens/

// Anchor resolution functions - currently not used during parsing
// pub(crate) use anchors::{collect_anchors, expand_merge_keys, replace_aliases};

#[cfg(test)]
pub(crate) use helpers::parse_quoted_scalar;
pub(crate) use helpers::{parse_comment, parse_error, peek_ahead_for_mapping_key};
pub(crate) use inline::{parse_inline_mapping, parse_inline_sequence};
pub(crate) use mapping::parse_mapping;
#[cfg(test)]
pub(crate) use scalar::parse_scalar;
pub(crate) use sequence::parse_sequence;
pub(crate) use value::parse_value;

use crate::io::traits::ISource;
use crate::nodes::node::BlockStyle;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::{DirectiveContext, parse_directives};
// use std::collections::HashMap;

use helpers::node_is_blank;
use helpers::skip_whitespace;

/// Parses multiple explicit keys for sets or mappings.
///
/// Handles the case where we have multiple consecutive lines starting with '?'
/// which typically represents a set with explicit key syntax.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The current indentation level for proper nesting
///
/// # Returns
///
/// Result containing a Mapping Node with null values, suitable for set conversion
fn parse_multiple_explicit_keys(
    source: &mut dyn ISource,
    indent_level: usize,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();

    while source.current() == Some('?') {
        // Skip the '?' character
        source.next();
        skip_whitespace(source);

        // Parse the key
        let key_node = if source.current() == Some('\n') || source.current().is_none() {
            // Empty key, skip this entry
            if source.current() == Some('\n') {
                source.next();
                skip_whitespace(source);
            }
            continue;
        } else {
            // Read the key content
            let key_content = crate::utils::read_line_trimmed_into_string(source);
            if source.current() == Some('\n') {
                source.next();
            }
            Node::Str(
                key_content,
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None,
            )
        };

        // For explicit keys without explicit values, the value is implicitly null
        pairs.push((key_node, Node::None));

        // Skip whitespace and check if we're still at the same indentation level
        skip_whitespace(source);
        let current_indent = source.get_current_indent_level();

        // If we're at a different indentation level or don't see another '?', stop
        if current_indent != indent_level || source.current() != Some('?') {
            break;
        }
    }

    Ok(Node::Mapping(pairs))
}

/// Parses the contents of a YAML document based on the current character and context.
///
/// Determines the appropriate parsing strategy based on the current character:
/// sequences (-), comments (#), inline mappings ({}), inline sequences ([]),
/// explicit mapping keys (?), block scalars (| or >), or regular mappings.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The indentation level for proper nesting
/// * `directives` - Directive context for tag resolution and version-specific parsing
///
/// # Returns
///
/// Result containing a Node or an error string
pub fn parse_document_contents(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    match source.current() {
        Some(c) if c == '-' => {
            // Check if this is a document marker (---)
            if helpers::peek_ahead_for_document_start_end(source, '-') {
                return Ok(Node::None);
            }
            let indent_level = source.get_current_indent_level();
            Ok(parse_sequence(source, indent_level, directives)?)
        }
        Some(c) if c == '.' => {
            // Check if this is a document end marker (...)
            if helpers::peek_ahead_for_document_start_end(source, '.') {
                return Ok(Node::None);
            }
            // Otherwise treat as regular content
            if peek_ahead_for_mapping_key(source) {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                Ok(parse_value(source, directives)?)
            }
        }
        Some(c) if c == '#' => {
            parse_comment(source);
            skip_whitespace(source);
            parse_document_contents(source, indent_level, directives)
        }
        Some(c) if c == '{' => Ok(parse_inline_mapping(source, directives)?),
        Some(c) if c == '[' => Ok(parse_inline_sequence(source, directives)?),
        // Support tagged values at the start of a nested block (e.g. indented "!!set" lines)
        Some(c) if c == '!' => {
            // Save state to check if this is a tagged mapping key
            let state = source.save_state();
            source.next(); // skip first !

            // Skip the tag
            while let Some(ch) = source.current() {
                if ch == ' ' || ch == '\t' || ch == '\n' {
                    break;
                }
                source.next();
            }
            crate::parser::document::helpers::skip_whitespace(source);

            // Check if what follows is a colon (indicating mapping key)
            let is_mapping_key = source.current() == Some(':');

            // Restore state and parse appropriately
            source.restore_state(state);

            if is_mapping_key {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                Ok(crate::parser::document::value::parse_value(
                    source, directives,
                )?)
            }
        }
        // Support anchors at document level (e.g. "&anchor key: value")
        Some(c) if c == '&' => {
            // Save state to check if this is an anchored mapping key
            let state = source.save_state();
            source.next(); // skip &

            // Collect anchor name
            let _anchor_name = crate::utils::collect_until(source, |c| {
                c == ' '
                    || c == '\t'
                    || c == '\n'
                    || c == '\r'
                    || c == '#'
                    || c == ','
                    || c == '['
                    || c == ']'
                    || c == '{'
                    || c == '}'
            });
            crate::parser::document::helpers::skip_whitespace(source);

            // Check if what follows looks like a mapping key
            let is_mapping = peek_ahead_for_mapping_key(source);

            // Restore state and parse appropriately
            source.restore_state(state);

            if is_mapping {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                Ok(crate::parser::document::value::parse_value(
                    source, directives,
                )?)
            }
        }
        // Support aliases at document level (e.g. "*anchor")
        Some(c) if c == '*' => Ok(crate::parser::document::value::parse_value(
            source, directives,
        )?),
        // Handle explicit value indicator (: value) with missing/null key
        Some(c) if c == ':' => {
            let current_indent = source.get_current_indent_level();
            let mut pairs: Vec<(Node, Node)> = Vec::new();

            // Parse all consecutive mapping entries with omitted keys at this indent level
            loop {
                // Check if we're still at the same indent and have a colon
                if source.get_current_indent_level() != current_indent
                    || source.current() != Some(':')
                {
                    break;
                }

                source.next(); // Skip ':'
                // Tabs are not allowed immediately after ':' indicator
                if source.current() == Some('\t') {
                    return Err(helpers::parse_error(
                        source,
                        "Tabs cannot be used as separation after explicit value indicator",
                    ));
                }
                skip_whitespace(source);

                // Parse the value
                let value_node = if source.current() == Some('\n') || source.current().is_none() {
                    // Just ":" with no value - null key and null value
                    if source.current() == Some('\n') {
                        source.next();
                    }
                    Node::None
                } else {
                    parse_value(source, directives)?
                };

                pairs.push((Node::None, value_node));

                // Move to next line
                crate::utils::skip_until_newline(source);
                if source.current() == Some('\n') {
                    source.next();
                }
                skip_whitespace(source);
            }

            Ok(Node::Mapping(pairs))
        }
        Some(c) if c == '?' => {
            // Check if we have multiple explicit keys at this indentation level (likely a set)
            let current_indent = source.get_current_indent_level();
            let state = source.save_state();

            // Look ahead to see if there are multiple '?' at the same indentation
            let mut has_multiple_explicit_keys = false;
            source.next(); // Skip first '?'
            crate::utils::skip_until_newline(source);
            if source.current() == Some('\n') {
                source.next();
            }
            skip_whitespace(source);

            if source.get_current_indent_level() == current_indent && source.current() == Some('?')
            {
                has_multiple_explicit_keys = true;
            }

            // Restore state to beginning of first '?'
            source.restore_state(state);

            if has_multiple_explicit_keys {
                // Parse multiple explicit keys (typically for sets)
                return Ok(parse_multiple_explicit_keys(source, current_indent)?);
            } else {
                // Original single explicit key logic
                source.next();
                // Tabs are not allowed immediately after '?' indicator
                if source.current() == Some('\t') {
                    return Err(helpers::parse_error(
                        source,
                        "Tabs cannot be used as separation after explicit key indicator",
                    ));
                }
                skip_whitespace(source);
                let mut key_node: Node;

                if source.current() == Some('[') {
                    key_node = parse_inline_sequence(source, directives)?;
                } else if source.current() == Some('-') {
                    let nested_indent = source.get_current_indent_level();
                    key_node = parse_sequence(source, nested_indent, directives)?;
                } else if matches!(source.current(), Some('|') | Some('>')) {
                    let is_folded = source.current() == Some('>');

                    // Use the consolidated block scalar parser
                    let content = block_scalar::parse_block_scalar(source, is_folded)?;

                    let mut escaped_key = content;
                    escaped_key.push_str("\\n");
                    key_node = Node::Str(
                        escaped_key,
                        crate::nodes::node::QuoteType::Double,
                        crate::nodes::node::BlockStyle::None,
                    );
                } else if matches!(
                    source.current(),
                    Some('"') | Some('\'') | Some('&') | Some('*') | Some('!')
                ) {
                    // Quoted strings, anchors, aliases, and tags need special parsing
                    key_node = crate::parser::document::value::parse_value(source, directives)?;
                } else if source.current() == Some('#') || source.current() == Some('\n') {
                    let st = source.save_state();
                    let _ = crate::utils::read_line_trimmed_into_string(source);
                    if source.current() == Some('\n') {
                        source.next();
                    }
                    skip_whitespace(source);
                    if source.current() == Some('-') {
                        let nested_indent = source.get_current_indent_level();
                        key_node = parse_sequence(source, nested_indent, directives)?;
                    } else {
                        source.restore_state(st);
                        key_node = Node::Str(
                            crate::utils::read_line_trimmed_into_string(source),
                            crate::nodes::node::QuoteType::Unquoted,
                            crate::nodes::node::BlockStyle::None,
                        );
                    }
                } else if source.current().is_some() {
                    key_node = Node::Str(
                        crate::utils::read_line_trimmed_into_string(source),
                        crate::nodes::node::QuoteType::Unquoted,
                        crate::nodes::node::BlockStyle::None,
                    );
                } else {
                    key_node = Node::Str(
                        String::new(),
                        crate::nodes::node::QuoteType::Unquoted,
                        crate::nodes::node::BlockStyle::None,
                    );
                }

                match key_node {
                    Node::Array(_) | Node::Mapping(_) => {
                        let inline = helpers::node_to_inline_string(&key_node);
                        key_node = Node::Str(
                            inline,
                            crate::nodes::node::QuoteType::Double,
                            crate::nodes::node::BlockStyle::None,
                        );
                    }
                    Node::Str(s, _qt, style) => {
                        let key_string = if matches!(style, BlockStyle::Literal) {
                            format!("{}\n", s)
                        } else {
                            s
                        };
                        key_node = Node::Str(
                            key_string,
                            crate::nodes::node::QuoteType::Double,
                            crate::nodes::node::BlockStyle::None,
                        );
                    }
                    other => {
                        let inline = helpers::node_to_inline_string(&other);
                        key_node = Node::Str(
                            inline,
                            crate::nodes::node::QuoteType::Double,
                            crate::nodes::node::BlockStyle::None,
                        );
                    }
                }

                let st_colon = source.save_state();
                let mut found_colon = false;
                loop {
                    skip_whitespace(source);
                    match source.current() {
                        Some(':') => {
                            source.next();
                            found_colon = true;
                            break;
                        }
                        Some('#') => {
                            parse_comment(source);
                            if source.current() == Some('\n') {
                                source.next();
                            }
                            continue;
                        }
                        Some('\n') => {
                            source.next();
                            continue;
                        }
                        Some(_) | None => break,
                    }
                }
                if !found_colon {
                    source.restore_state(st_colon);
                    if source.current() == Some('\n') {
                        source.next();
                    }
                    loop {
                        skip_whitespace(source);
                        if source.current() == Some(':') {
                            break;
                        }
                        if source.current().is_none() {
                            break;
                        }
                        crate::utils::skip_until_newline(source);
                        if source.current().is_none() {
                            break;
                        }
                    }
                }
                if source.current() == Some(':') {
                    source.next();
                }
                skip_whitespace(source);
                let mut value_node = match source.current() {
                    Some('[') => parse_inline_sequence(source, directives)?,
                    Some('{') => parse_inline_mapping(source, directives)?,
                    Some('-') => {
                        let nested_indent = source.get_current_indent_level();
                        parse_sequence(source, nested_indent, directives)?
                    }
                    Some(_) => parse_value(source, directives)?,
                    None => {
                        // EOF after explicit key means implicit null value
                        Node::None
                    }
                };

                if matches!(value_node, Node::None) {
                    let st_peek = source.save_state();
                    crate::utils::skip_whitespace_and_comments(source);
                    if source.current() == Some('-') {
                        let nested_indent = source.get_current_indent_level();
                        value_node = parse_sequence(source, nested_indent, directives)?;
                    } else {
                        source.restore_state(st_peek);
                    }
                }

                let mut pairs: Vec<(Node, Node)> = Vec::new();
                pairs.push((key_node, value_node));
                Ok(Node::Mapping(pairs))
            }
        }
        Some(c) if c.is_alphanumeric() => {
            if peek_ahead_for_mapping_key(source) {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else if indent_level > 0 {
                let base_indent = source.get_current_indent_level();
                let mut parts: Vec<String> = Vec::new();
                loop {
                    let line = crate::utils::read_line_trimmed_into_string(source);
                    if !line.is_empty() {
                        parts.push(line);
                    }
                    if source.current() == Some('\n') {
                        source.next();
                    }
                    let st = source.save_state();
                    skip_whitespace(source);
                    let cur_indent = source.get_current_indent_level();
                    let next_char = source.current();
                    source.restore_state(st);
                    if next_char.is_none() || cur_indent < base_indent {
                        break;
                    }
                    if matches!(
                        next_char,
                        Some('-') | Some('{') | Some('[') | Some('?') | Some('#')
                    ) {
                        break;
                    }
                }
                let joined = parts.join(" ");
                Ok(Node::Str(
                    joined,
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::None,
                ))
            } else {
                // At root level (indent_level == 0) without a colon, parse as multiline plain scalar
                let mut lines = Vec::new();
                let start_indent = source.get_current_indent_level();

                loop {
                    // Read the current line
                    let line = crate::utils::read_line_trimmed_into_string(source);
                    if !line.is_empty() {
                        lines.push(line);
                    }

                    // Skip newline if present
                    if source.current() == Some('\n') {
                        source.next();
                    }

                    // Check what comes next
                    if !source.more() {
                        break;
                    }

                    // Skip whitespace to check indent of next line
                    skip_whitespace(source);

                    if !source.more() {
                        break;
                    }

                    let line_indent = source.get_current_indent_level();
                    let ch = source.current();

                    // Stop if we hit a document marker
                    if ch == Some('-') && helpers::peek_ahead_for_document_start_end(source, '-') {
                        break;
                    }
                    if ch == Some('.') && helpers::peek_ahead_for_document_start_end(source, '.') {
                        break;
                    }

                    // Stop if we hit a directive
                    if ch == Some('%') && line_indent == 0 {
                        break;
                    }

                    // Stop on dedent (but not at indent 0, where all lines are the same)
                    if start_indent > 0 && line_indent < start_indent {
                        break;
                    }

                    // Stop on comment
                    if ch == Some('#') {
                        break;
                    }

                    // Continue if at same or greater indent
                    if line_indent >= start_indent {
                        continue;
                    }

                    // Otherwise stop
                    break;
                }

                let scalar_value = lines.join(" ");
                Ok(Node::Str(
                    scalar_value,
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::None,
                ))
            }
        }
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(source, indent_level, directives)?)
        }
        Some('\0') => {
            source.next();
            Ok(parse_document_contents(source, indent_level, directives)?)
        }
        Some(c) if matches!(c, '<' | '>' | '"' | '\'' | '|') => {
            if matches!(source.current(), Some('"') | Some('\''))
                && peek_ahead_for_mapping_key(source)
            {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                Ok(parse_value(source, directives)?)
            }
        }
        Some(c) => Err(helpers::parse_error(
            source,
            &format!(
                "{}{}",
                crate::error::messages::ERR_UNEXPECTED_CHAR_PREFIX,
                c
            ),
        )),
        None => Ok(Node::None),
    }
}

/// Parses a single YAML document from the source.
///
/// Processes document content while handling document start/end markers (--- and ...),
/// comments, and various node types. Collects all document nodes and performs
/// post-processing including anchor resolution and merge key expansion.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The indentation level for the document
/// * `directives` - Directive context for tag resolution and version-specific parsing
///
/// # Returns
///
/// Result containing a Document Node or an error string
pub fn parse_document(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    skip_whitespace(source);

    let mut document_nodes = Vec::new();

    while let Some(c) = source.current() {
        if (c == '-' || c == '.')
            && crate::parser::document::helpers::peek_ahead_for_document_start_end(source, c)
        {
            // Found document marker - just break, let parse() handle it
            break;
        }

        match c {
            '#' => {
                parse_comment(source);
                skip_whitespace(source);
                continue;
            }
            '%' => {
                // Only treat % as directive separator if we're at document boundary (no content yet)
                // Otherwise % can be valid content in plain scalars
                if document_nodes.is_empty() {
                    break;
                } else {
                    // Already have content, so % should have been consumed as part of it
                    // If we're here, there's a parsing issue
                    return Err(helpers::parse_error(
                        source,
                        "Unexpected % after document content (directives must appear before content)",
                    ));
                }
            }
            _ => {
                let node = parse_document_contents(source, indent_level, directives)?;
                if !node_is_blank(&node) {
                    document_nodes.push(node);
                }
            }
        }
    }

    let mut normalized_nodes: Vec<Node> = Vec::new();
    let mut i = 0usize;
    while i < document_nodes.len() {
        if i + 1 < document_nodes.len() {
            if let Node::Mapping(pairs) = &document_nodes[i] {
                if pairs.len() == 1 && matches!(pairs[0].1, Node::None) {
                    if let Node::Array(arr) = &document_nodes[i + 1] {
                        let key = pairs[0].0.clone();
                        normalized_nodes.push(Node::Mapping(vec![(key, Node::Array(arr.clone()))]));
                        i += 2;
                        continue;
                    }
                }
            }
        }
        normalized_nodes.push(document_nodes[i].clone());
        i += 1;
    }

    let doc_node = Document(normalized_nodes);

    // TODO: Anchor/alias resolution should be optional or done separately
    // Many use cases (like test suites) expect the raw parse tree with
    // anchors and aliases preserved, not automatically resolved.
    // let mut anchors: HashMap<String, Node> = HashMap::new();
    // collect_anchors(&doc_node, &mut anchors)?;

    // expand_merge_keys(&mut doc_node, &anchors)?;
    // replace_aliases(&mut doc_node, &anchors)?;

    Ok(doc_node)
}

/// Main entry point for parsing YAML content from a source.
///
/// Parses one or more YAML documents from the source, handling document
/// separators and creating a Documents node containing all parsed documents.
/// Empty or blank documents are filtered out automatically.
///
/// Also parses directives (%YAML and %TAG) that appear before each document.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// Result containing a Documents Node with all parsed documents or an error string
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    let mut docs: Vec<Node> = Vec::new();

    while source.more() {
        // Parse directives before this document
        let directives = parse_directives(source)?;

        // Track if we have explicit directives
        let has_explicit_directives =
            directives.yaml_version.is_some() || directives.tag_prefixes.len() > 2;

        // Check for document start marker (---)
        let has_document_marker = helpers::peek_ahead_for_document_start_end(source, '-');
        if has_document_marker {
            source.next();
            source.next();
            source.next();

            // After ---, only whitespace/comments allowed until end of line
            // Exception: block scalar indicators (>, |) and tags (!) are allowed
            skip_whitespace(source);
            if let Some(c) = source.current() {
                // Allow: newline, carriage return, comment, block scalar indicators, tags
                // Tags can appear after --- to apply to the document (e.g., --- !<tag>)
                if c != '\n' && c != '\r' && c != '#' && c != '>' && c != '|' && c != '!' {
                    // Check if it's the start of a mapping key pattern (key:)
                    // Save state to check ahead
                    let state = source.save_state();
                    let mut found_colon = false;
                    while let Some(ch) = source.current() {
                        if ch == ':' {
                            found_colon = true;
                            break;
                        }
                        if ch == '\n' || ch == '\r' {
                            break;
                        }
                        source.next();
                    }
                    source.restore_state(state);

                    // If we found a colon on same line, it's invalid mapping on --- line
                    if found_colon {
                        return Err(helpers::parse_error(
                            source,
                            "Document start marker (---) must be on its own line",
                        ));
                    }
                }
            }
            // Skip comments and move to next line if appropriate
            if source.current() == Some('#') {
                helpers::parse_comment(source);
            }
            if source.current() == Some('\n') || source.current() == Some('\r') {
                source.next();
            }
        }

        // If we have explicit directives but no document marker, that's an error
        if has_explicit_directives && !has_document_marker {
            return Err(helpers::parse_error(
                source,
                "Directives require a document (use --- to start document)",
            ));
        }

        // Parse the document with directive context
        let document = parse_document(source, 0, &directives);
        match document {
            Ok(doc) => {
                let is_blank_doc = match &doc {
                    Document(nodes) => nodes.iter().all(node_is_blank),
                    _ => false,
                };

                // Empty documents are valid after --- marker
                if !is_blank_doc {
                    docs.push(doc)
                }
            }
            Err(err) => return Err(err),
        }

        // Check for document end marker (...)
        skip_whitespace(source);
        let has_document_end = helpers::peek_ahead_for_document_start_end(source, '.');
        if has_document_end {
            source.next();
            source.next();
            source.next();

            // Check for invalid content after document end marker
            skip_whitespace(source);
            if let Some(c) = source.current() {
                // Allow newline, carriage return, comments, and directives (%)
                if c != '\n' && c != '\r' && c != '#' && c != '%' {
                    // There's non-whitespace, non-comment, non-directive content after ...
                    return Err(parse_error(
                        source,
                        "Invalid content after document end marker (...)",
                    ));
                }
            }

            if source.current() == Some('\n') {
                source.next();
            }
        }

        // If no more content, stop
        if !source.more() {
            break;
        }

        // Check if next content starts with directive without document-end marker
        // Peek ahead to see if we're about to parse directives for next document
        skip_whitespace(source);
        if !has_document_end && source.current() == Some('%') {
            // We have a directive after document content but no ... marker
            return Err(parse_error(
                source,
                "Directive requires document-end marker (...) before starting new document",
            ));
        }
    }

    if docs.is_empty() {
        docs.push(Document(Vec::new()))
    }
    Ok(Node::Documents(docs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_parse_scalar() {
        let directives = crate::parser::directives::DirectiveContext::new();
        assert_eq!(parse_scalar("null", &directives), Node::None);
        assert_eq!(parse_scalar("~", &directives), Node::None);
        assert_eq!(parse_scalar("true", &directives), Node::Boolean(true));
        assert_eq!(parse_scalar("false", &directives), Node::Boolean(false));
        assert_eq!(
            parse_scalar("42", &directives),
            Node::Number(crate::nodes::node::Numeric::Integer(42))
        );
        assert_eq!(
            parse_scalar("3.14", &directives),
            Node::Number(crate::nodes::node::Numeric::Float(3.14))
        );
        assert_eq!(
            parse_scalar("hello", &directives),
            Node::Str(
                "hello".to_string(),
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None
            )
        );
        assert_eq!(
            parse_scalar("#comment", &directives),
            Node::Str(
                "#comment".to_string(),
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None
            )
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_basic() {
        let mut source = Buffer::new(b"key: value");
        assert_eq!(source.get_current_indent_level(), 0);
        assert!(peek_ahead_for_mapping_key(&mut source));
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_no_colon() {
        let mut source = Buffer::new(b"key value");
        assert!(!peek_ahead_for_mapping_key(&mut source));
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_colon_after_newline() {
        let mut source = Buffer::new(b"key\n: value");
        assert!(!peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_spaces_before_colon() {
        let mut source = Buffer::new(b"key   : value");
        assert!(peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_empty() {
        let mut source = Buffer::new(b"");
        assert!(!peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_parse_quoted_scalar_single_and_double() {
        let mut s1 = Buffer::new(b"'single''quote'");
        let r1 = parse_quoted_scalar(&mut s1).unwrap();
        assert_eq!(r1, "'single''quote'");

        let mut s2 = Buffer::new(b"\"double\\\"quote\"");
        let r2 = parse_quoted_scalar(&mut s2).unwrap();
        assert_eq!(r2, "\"double\\\"quote\"");
    }

    #[test]
    fn test_parse_inline_sequence_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"[1, 'two', 3]");
        let node = parse_inline_sequence(&mut src, &directives).unwrap();
        assert!(matches!(node, Node::Array(_)));
        if let Node::Array(items) = node {
            assert_eq!(items.len(), 3);
            assert!(matches!(
                items[0],
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
            assert!(matches!(
                items[1],
                Node::Str(_, crate::nodes::node::QuoteType::Single, _)
            ));
            assert!(matches!(
                items[2],
                Node::Number(crate::nodes::node::Numeric::Integer(3))
            ));
        }

        let mut empty = Buffer::new(b"[]");
        let node = parse_inline_sequence(&mut empty, &directives).unwrap();
        assert!(matches!(node, Node::Array(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_inline_mapping_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"{key1: 1, 'key2': \"two\"}");
        let node = parse_inline_mapping(&mut src, &directives).unwrap();
        assert!(matches!(node, Node::Mapping(_)));
        if let Node::Mapping(pairs) = node {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(
                pairs[0].0,
                Node::Str(_, crate::nodes::node::QuoteType::Unquoted, _)
            ));
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
            assert!(matches!(
                pairs[1].0,
                Node::Str(_, crate::nodes::node::QuoteType::Single, _)
            ));
            assert!(matches!(pairs[1].1, Node::Str(_, _, _)));
        }

        let mut empty = Buffer::new(b"{}");
        let node = parse_inline_mapping(&mut empty, &directives).unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_comment_trims_hash_and_newline() {
        let mut src = Buffer::new(b"# Hello world  \n");
        let text = parse_comment(&mut src);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_parse_value_alias_and_anchor() {
        let directives = DirectiveContext::new();
        let mut a = Buffer::new(b"*myalias");
        let n = parse_value(&mut a, &directives).unwrap();
        assert!(matches!(n, Node::Alias(ref name) if name == "myalias"));

        let mut b = Buffer::new(b"&aname 42");
        let n = parse_value(&mut b, &directives).unwrap();
        if let Node::Anchored(inner, name) = n {
            assert_eq!(*name, "aname".to_string());
            assert!(matches!(
                *inner,
                Node::Number(crate::nodes::node::Numeric::Integer(42))
            ));
        } else {
            panic!("expected Anchored node");
        }
    }

    #[test]
    fn test_parse_document_contents_empty_line() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"key: value\n\n");
        let n = parse_document_contents(&mut src, 0, &directives).unwrap();
        assert!(matches!(n, Node::Mapping(_)));
    }
}
