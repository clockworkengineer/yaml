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

pub(crate) use helpers::parse_error;
pub(crate) use inline::{parse_inline_mapping, parse_inline_sequence};
pub(crate) use mapping::parse_mapping;
#[cfg(test)]
#[cfg(test)]
pub(crate) use scalar::parse_scalar_with_tokens;
pub(crate) use sequence::parse_sequence;
pub(crate) use value::parse_value;

use crate::io::traits::ISource;
use crate::nodes::node::BlockStyle;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::{DirectiveContext, parse_directives};
// use std::collections::HashMap;

use helpers::node_is_blank;

/// Token-based lookahead to determine if the current position starts a mapping key (key: ...)
/// Scans tokens until end-of-line and returns true if a colon token appears at the same nesting level
/// and not inside flow collections.

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
        let directives_local = crate::parser::directives::DirectiveContext::new();
        {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, &directives_local)?;
            // Note: use local stream only for skipping; actual parsing below uses provided directives
            stream.skip_whitespace_and_comments()?;
        }

        // Parse the key via tokens; normalize to string for set semantics
        let mut stream = crate::parser::token_stream::TokenStream::new(source, &directives_local)?;
        let mut key_node = match stream.current() {
            Some(crate::parser::lexer::Token::Newline) | None => {
                // Empty key, skip this entry
                if source.current() == Some('\n') {
                    source.next();
                }
                // Continue to next explicit key
                continue;
            }
            _ => crate::parser::document::tokens::value::parse_value_with_tokens(
                &mut stream,
                &directives_local,
            )?,
        };

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

        // For explicit keys without explicit values, the value is implicitly null
        pairs.push((key_node, Node::None));

        // Skip whitespace and check if we're still at the same indentation level
        crate::utils::skip_whitespace_and_comments(source);
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
    // Fast token-dispatch for block constructs to prefer tokenized paths
    {
        let st = source.save_state();
        if let Ok(ts) = crate::parser::token_stream::TokenStream::new(source, directives) {
            match ts.current() {
                Some(crate::parser::lexer::Token::Indent(lvl)) => {
                    let level_val = *lvl;
                    // Route to tokenized block mapping when encountering indentation
                    source.restore_state(st);
                    let mut stream =
                        crate::parser::token_stream::TokenStream::new(source, directives)?;
                    return crate::parser::document::mapping_tokens::parse_mapping_with_tokens(
                        &mut stream,
                        level_val,
                        directives,
                    );
                }
                Some(crate::parser::lexer::Token::Dash) => {
                    // Route to tokenized block sequence when encountering dash at line start
                    source.restore_state(st);
                    let seq_indent = source.get_current_indent_level();
                    let mut stream =
                        crate::parser::token_stream::TokenStream::new(source, directives)?;
                    return crate::parser::document::sequence_tokens::parse_sequence_with_tokens(
                        &mut stream,
                        seq_indent,
                        directives,
                    );
                }
                _ => {
                    // continue with existing char-dispatch below
                    source.restore_state(st);
                }
            }
        } else {
            source.restore_state(st);
        }
    }
    match source.current() {
        Some(c) if c == '-' => {
            // Check if this is a document marker (---)
            let seq_indent = source.get_current_indent_level();
            // Prefer token-based detection for document start
            let is_doc_start = {
                let st = source.save_state();
                let ts = crate::parser::token_stream::TokenStream::new(source, directives)?;
                let res = matches!(
                    ts.current(),
                    Some(crate::parser::lexer::Token::DocumentStart)
                );
                source.restore_state(st);
                res
            };
            if is_doc_start {
                if seq_indent == 0 {
                    return Ok(Node::None);
                } else {
                    // Not at indent 0, treat as unquoted string
                    let s = crate::utils::read_line_trimmed_into_string(source);
                    return Ok(Node::Str(
                        s,
                        crate::nodes::node::QuoteType::Unquoted,
                        crate::nodes::node::BlockStyle::None,
                    ));
                }
            }
            // Tokenize to confirm dash is a sequence indicator at this position
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::Dash) => {
                    if seq_indent < indent_level {
                        return Err(format!(
                            "Sequence item at invalid indentation: expected >= {}, got {}",
                            indent_level, seq_indent
                        ));
                    }
                    // Use token-based sequence parser starting from the same stream (dash present)
                    Ok(
                        crate::parser::document::sequence_tokens::parse_sequence_with_tokens(
                            &mut stream,
                            seq_indent,
                            directives,
                        )?,
                    )
                }
                _ => {
                    // Fallback: treat as value if not a sequence dash token
                    Ok(parse_value(source, directives)?)
                }
            }
        }
        Some(c) if c == '.' => {
            // Check if this is a document end marker (...)
            let map_indent = source.get_current_indent_level();
            let is_doc_end = {
                let st = source.save_state();
                let ts = crate::parser::token_stream::TokenStream::new(source, directives)?;
                let res = matches!(ts.current(), Some(crate::parser::lexer::Token::DocumentEnd));
                source.restore_state(st);
                res
            };
            if is_doc_end {
                if map_indent == 0 {
                    return Ok(Node::None);
                } else {
                    // Not at indent 0, treat as unquoted string
                    let s = crate::utils::read_line_trimmed_into_string(source);
                    return Ok(Node::Str(
                        s,
                        crate::nodes::node::QuoteType::Unquoted,
                        crate::nodes::node::BlockStyle::None,
                    ));
                }
            }
            if map_indent < indent_level {
                return Err(format!(
                    "Mapping key at invalid indentation: expected >= {}, got {}",
                    indent_level, map_indent
                ));
            }
            if helpers::peek_ahead_for_mapping_key(source, directives) {
                Ok(parse_mapping(source, map_indent, directives)?)
            } else {
                Ok(parse_value(source, directives)?)
            }
        }
        Some(c) if c == '#' => {
            // Use token stream to skip comments and whitespace uniformly
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
            stream.skip_whitespace_and_comments()?;
            parse_document_contents(source, indent_level, directives)
        }
        Some(c) if c == '{' => {
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
            Ok(parse_inline_mapping(&mut stream, directives)?)
        }
        Some(c) if c == '[' => {
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
            Ok(parse_inline_sequence(&mut stream, directives)?)
        }
        // Support tagged values or tagged keys using TokenStream
        Some(c) if c == '!' => {
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
            // If a tag token appears and is followed by a colon at same level, treat as mapping key
            match stream.current() {
                Some(crate::parser::lexer::Token::Tag(_)) => {
                    stream.next()?;
                    stream.skip_whitespace_and_comments()?;
                    let is_mapping_key =
                        matches!(stream.current(), Some(crate::parser::lexer::Token::Colon));
                    if is_mapping_key {
                        Ok(parse_mapping(source, indent_level, directives)?)
                    } else {
                        Ok(
                            crate::parser::document::tokens::value::parse_value_with_tokens(
                                &mut stream,
                                directives,
                            )?,
                        )
                    }
                }
                _ => Ok(
                    crate::parser::document::tokens::value::parse_value_with_tokens(
                        &mut stream,
                        directives,
                    )?,
                ),
            }
        }
        // Support anchors using TokenStream
        Some(c) if c == '&' => {
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::Anchor(_)) => {
                    stream.next()?;
                    stream.skip_whitespace_and_comments()?;
                    let is_mapping_key =
                        matches!(stream.current(), Some(crate::parser::lexer::Token::Colon));
                    if is_mapping_key {
                        Ok(parse_mapping(source, indent_level, directives)?)
                    } else {
                        Ok(
                            crate::parser::document::tokens::value::parse_value_with_tokens(
                                &mut stream,
                                directives,
                            )?,
                        )
                    }
                }
                _ => Ok(
                    crate::parser::document::tokens::value::parse_value_with_tokens(
                        &mut stream,
                        directives,
                    )?,
                ),
            }
        }
        // Support aliases at document level (e.g. "*anchor")
        Some(c) if c == '*' => Ok(crate::parser::document::value::parse_value(
            source, directives,
        )?),
        // Handle explicit value indicator (: value) with missing/null key using tokens
        Some(c) if c == ':' => {
            let current_indent = source.get_current_indent_level();
            let mut pairs: Vec<(Node, Node)> = Vec::new();
            loop {
                if source.get_current_indent_level() != current_indent
                    || source.current() != Some(':')
                {
                    break;
                }
                source.next();
                if source.current() == Some('\t') {
                    return Err(helpers::parse_error(
                        source,
                        "Tabs cannot be used as separation after explicit value indicator",
                    ));
                }
                crate::utils::skip_whitespace_and_comments(source);

                let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
                let value_node = match stream.current() {
                    Some(crate::parser::lexer::Token::Newline) | None => Node::None,
                    _ => crate::parser::document::tokens::value::parse_value_with_tokens(
                        &mut stream,
                        directives,
                    )?,
                };

                pairs.push((Node::None, value_node));

                crate::utils::skip_until_newline(source);
                if source.current() == Some('\n') {
                    source.next();
                }
                crate::utils::skip_whitespace_and_comments(source);
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
            crate::utils::skip_whitespace_and_comments(source);

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
                // Refactored single explicit key logic to use TokenStream-based parsing
                source.next();
                if source.current() == Some('\t') {
                    return Err(helpers::parse_error(
                        source,
                        "Tabs cannot be used as separation after explicit key indicator",
                    ));
                }
                let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
                stream.skip_whitespace_and_comments()?;
                let mut key_node = crate::parser::document::tokens::value::parse_value_with_tokens(
                    &mut stream,
                    directives,
                )?;

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
                    // Token-based skip over whitespace/comments/newlines
                    {
                        let mut stream =
                            crate::parser::token_stream::TokenStream::new(source, directives)?;
                        stream.skip_whitespace_and_comments()?;
                    }
                    match source.current() {
                        Some(':') => {
                            source.next();
                            found_colon = true;
                            break;
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
                        {
                            let mut stream =
                                crate::parser::token_stream::TokenStream::new(source, directives)?;
                            stream.skip_whitespace_and_comments()?;
                        }
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
                {
                    let mut stream =
                        crate::parser::token_stream::TokenStream::new(source, directives)?;
                    stream.skip_whitespace_and_comments()?;
                }
                let mut value_node = match source.current() {
                    Some('[') => {
                        let mut stream =
                            crate::parser::token_stream::TokenStream::new(source, directives)?;
                        parse_inline_sequence(&mut stream, directives)?
                    }
                    Some('{') => {
                        let mut stream =
                            crate::parser::token_stream::TokenStream::new(source, directives)?;
                        parse_inline_mapping(&mut stream, directives)?
                    }
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
            if helpers::peek_ahead_for_mapping_key(source, directives) {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                // If a plain word is followed by a newline and greater indentation
                // without a colon, this is likely a missing colon error.
                let state = source.save_state();
                // Create token stream to align with lexer boundaries (no assignment needed)
                let mut ts = crate::parser::token_stream::TokenStream::new(source, directives)?;
                match ts.current() {
                    Some(crate::parser::lexer::Token::Plain(_)) => {}
                    _ => {
                        // Fallback to scalar consumption to normalize position
                        let _ = ts.consume_plain_scalar()?;
                    }
                };
                // After consuming the current plain scalar, check newline + indent
                // Note: ts has advanced; reflect in source by restoring then consuming characters
                source.restore_state(state);
                // Read until end of line
                crate::utils::skip_until_newline(source);
                // If newline present, move to next char
                if source.current() == Some('\n') {
                    source.next();
                }
                let next_indent = source.get_current_indent_level();
                let st_after_nl = source.save_state();
                // Skip horizontal spaces
                while matches!(source.current(), Some(' ')) {
                    source.next();
                }
                let next_char = source.current();
                source.restore_state(st_after_nl);

                if next_indent > indent_level {
                    // If next significant char doesn't start a valid nested construct,
                    // report a missing colon.
                    if !matches!(
                        next_char,
                        Some('-') | Some('?') | Some('[') | Some('{') | Some('#')
                    ) {
                        return Err(parse_error(source, "Mapping key without colon"));
                    }
                }

                // Otherwise, treat as plain scalar
                let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
                let s = stream.consume_plain_scalar()?;
                Ok(Node::Str(
                    s,
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
                && helpers::peek_ahead_for_mapping_key(source, directives)
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
    #[cfg(feature = "debug-trace")]
    log::debug!("parse_document: start at indent {}", indent_level);
    crate::utils::skip_whitespace_and_comments(source);

    let mut document_nodes = Vec::new();

    while let Some(c) = source.current() {
        // Break if at document marker tokens
        let is_marker = {
            let st = source.save_state();
            let ts = crate::parser::token_stream::TokenStream::new(source, directives)?;
            let res = matches!(
                ts.current(),
                Some(
                    crate::parser::lexer::Token::DocumentStart
                        | crate::parser::lexer::Token::DocumentEnd
                )
            );
            source.restore_state(st);
            res
        };
        if is_marker {
            // Found document marker - just break, let parse() handle it
            break;
        }

        match c {
            '#' => {
                let mut stream = crate::parser::token_stream::TokenStream::new(source, directives)?;
                stream.skip_whitespace_and_comments()?;
                continue;
            }
            '%' => {
                // Treat % as a directive boundary; break to allow parse() to handle directives
                break;
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

    #[cfg(feature = "debug-trace")]
    {
        // Safe to compute lightweight stats for debug purposes
        let node_count = match &doc_node {
            Document(nodes) => nodes.len(),
            _ => 0,
        };
        log::debug!(
            "parse_document: completed with {} top-level node(s)",
            node_count
        );
    }
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
    #[cfg(feature = "debug-trace")]
    log::debug!("parse: begin stream");
    let mut docs: Vec<Node> = Vec::new();

    while source.more() {
        // Ensure we're positioned at meaningful content before checks
        crate::utils::skip_whitespace_and_comments(source);
        // Parse directives before this document
        let directives = parse_directives(source)?;

        // Track if we have explicit directives
        let has_explicit_directives =
            directives.yaml_version.is_some() || directives.tag_prefixes.len() > 2;

        // If we have explicit directives, require a following document with content
        if has_explicit_directives {
            let st = source.save_state();
            let mut ts = crate::parser::token_stream::TokenStream::new(source, &directives)?;
            // Skip trivia
            ts.skip_whitespace_and_comments()?;
            match ts.current() {
                // A document start marker is acceptable only if followed by real content
                Some(crate::parser::lexer::Token::DocumentStart) => {
                    // Advance past '---' and check for end-of-line content rules in existing logic
                }
                // A document end or EOF immediately after directives is invalid
                Some(crate::parser::lexer::Token::DocumentEnd)
                | Some(crate::parser::lexer::Token::Eof)
                | None => {
                    source.restore_state(st);
                    return Err(parse_error(
                        source,
                        "Directive must be followed by a document",
                    ));
                }
                _ => {
                    // Proceed; there is content following directives
                }
            }
            source.restore_state(st);
        }

        // Check for document start marker (---)
        let has_document_marker = {
            let st = source.save_state();
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives)?;
            let res = matches!(
                ts.current(),
                Some(crate::parser::lexer::Token::DocumentStart)
            );
            source.restore_state(st);
            res
        };
        if has_document_marker {
            source.next();
            source.next();
            source.next();

            // After ---, only whitespace/comments allowed until end of line
            // Exception: block scalar indicators (>, |) and tags (!) are allowed
            crate::utils::skip_whitespace_and_comments(source);
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

        // Allow explicit directives without a document start marker.
        // Per YAML spec, directives may appear at the top and apply to the following document,
        // with or without an explicit '---'. Do not error here.

        // Parse the document with directive context
        let document = parse_document(source, 0, &directives);
        match document {
            Ok(doc) => {
                // Count all documents, including empty ones, to match stream semantics
                docs.push(doc)
            }
            Err(err) => return Err(err),
        }

        // Check for document end marker (...)
        crate::utils::skip_whitespace_and_comments(source);
        let has_document_end = {
            let st = source.save_state();
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives)?;
            let res = matches!(ts.current(), Some(crate::parser::lexer::Token::DocumentEnd));
            source.restore_state(st);
            res
        };
        if has_document_end {
            source.next();
            source.next();
            source.next();

            // Check for invalid content after document end marker
            crate::utils::skip_whitespace_and_comments(source);
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

        // If no more content after handling markers, stop
        if !source.more() {
            break;
        }

        // Allow directives to start the next document even without explicit document-end marker.
        // YAML parsers may accept directives at document boundaries without requiring '...'.

        // Continue to parse next document
    }

    if docs.is_empty() {
        docs.push(Document(Vec::new()))
    }
    #[cfg(feature = "debug-trace")]
    log::debug!("parse: end stream with {} document(s)", docs.len());
    Ok(Node::Documents(docs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::document::helpers::parse_comment;

    #[test]
    fn test_parse_scalar_with_tokens() {
        use crate::io::sources::buffer::Buffer;
        let directives = crate::parser::directives::DirectiveContext::new();

        // Helper to parse a single-token scalar through TokenStream
        fn parse_one(
            input: &str,
            directives: &crate::parser::directives::DirectiveContext,
        ) -> Result<Node, String> {
            let mut src = Buffer::new(input.as_bytes());
            let mut stream = crate::parser::token_stream::TokenStream::new(&mut src, directives)?;
            parse_scalar_with_tokens(&mut stream, directives, 0)
        }

        assert_eq!(parse_one("null", &directives), Ok(Node::None));
        assert_eq!(parse_one("~", &directives), Ok(Node::None));
        assert_eq!(parse_one("true", &directives), Ok(Node::Boolean(true)));
        assert_eq!(parse_one("false", &directives), Ok(Node::Boolean(false)));
        assert_eq!(
            parse_one("42", &directives),
            Ok(Node::Number(crate::nodes::node::Numeric::Integer(42)))
        );
        assert_eq!(
            parse_one("3.14", &directives),
            Ok(Node::Number(crate::nodes::node::Numeric::Float(3.14)))
        );
        assert_eq!(
            parse_one("hello", &directives),
            Ok(Node::Str(
                "hello".to_string(),
                crate::nodes::node::QuoteType::Unquoted,
                crate::nodes::node::BlockStyle::None
            ))
        );
        // In token-based parsing, leading '#' starts a comment, so quote to treat as scalar
        assert_eq!(
            parse_one("'#comment'", &directives),
            Ok(Node::Str(
                "#comment".to_string(),
                crate::nodes::node::QuoteType::Single,
                crate::nodes::node::BlockStyle::None
            ))
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_basic() {
        let mut source = Buffer::new(b"key: value");
        assert_eq!(source.get_current_indent_level(), 0);
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_no_colon() {
        let mut source = Buffer::new(b"key value");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            !crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_colon_after_newline() {
        let mut source = Buffer::new(b"key\n: value");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            !crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_spaces_before_colon() {
        let mut source = Buffer::new(b"key   : value");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_empty() {
        let mut source = Buffer::new(b"");
        let directives = crate::parser::directives::DirectiveContext::new();
        assert!(
            !crate::parser::document::helpers::peek_ahead_for_mapping_key(&mut source, &directives)
        );
    }

    #[test]
    fn test_parse_inline_sequence_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"[1, 'two', 3]");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut src, &directives).unwrap();
        let node = parse_inline_sequence(&mut stream, &directives).unwrap();
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
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut empty, &directives).unwrap();
        let node = parse_inline_sequence(&mut stream, &directives).unwrap();
        assert!(matches!(node, Node::Array(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_inline_mapping_simple_and_empty() {
        let directives = DirectiveContext::new();
        let mut src = Buffer::new(b"{key1: 1, 'key2': \"two\"}");
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut src, &directives).unwrap();
        let node = parse_inline_mapping(&mut stream, &directives).unwrap();
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
        let mut stream =
            crate::parser::token_stream::TokenStream::new(&mut empty, &directives).unwrap();
        let node = parse_inline_mapping(&mut stream, &directives).unwrap();
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
