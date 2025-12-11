use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::parse_directives;

use crate::parser::document::document_parser::parse_document;
use crate::parser::document::helpers;

/// Checks for and processes the document start marker (---).
/// Returns an error if invalid content is found after the marker.
fn parse_document_markers(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<(), String> {
    // Check for document start marker (---)
    let has_document_marker = {
        let st = source.save_state();
        let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
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
        if let Some(c) = source.current() {
            if c != '\n' && c != '\r' && c != '#' && c != '>' && c != '|' && c != '!' && c != '-' {
                return Err(helpers::parse_error(
                    source,
                    "YAML 1.2: Document start marker (---) must be on its own line, except for comments, block scalar indicators (|, >), or tags (!). No mapping keys or values allowed on the same line as ---.",
                ));
            }
        }
        if source.current() == Some('#') {
            helpers::parse_comment(source);
        }
        if source.current() == Some('\n') || source.current() == Some('\r') {
            source.next();
        }
    }
    Ok(())
}

/// Checks for and processes the document end marker (...).
/// Returns an error if invalid content is found after the marker.
fn parse_document_end_marker(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<(), String> {
    crate::utils::skip_whitespace_and_comments(source);
    let has_document_end = {
        let st = source.save_state();
        let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
        let res = matches!(ts.current(), Some(crate::parser::lexer::Token::DocumentEnd));
        source.restore_state(st);
        res
    };
    if has_document_end {
        source.next();
        source.next();
        source.next();
        crate::utils::skip_whitespace_and_comments(source);
        if let Some(c) = source.current() {
            if c != '\n' && c != '\r' && c != '#' && c != '%' && c != '-' {
                return Err(helpers::parse_error(
                    source,
                    "Invalid content after document end marker (...)",
                ));
            }
        }
        if source.current() == Some('\n') {
            source.next();
        }
    }
    Ok(())
}

/// Checks for explicit directives and ensures a document follows them.
/// Returns an error if directives are not followed by a document.
fn check_explicit_directives(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<(), String> {
    let has_explicit_directives =
        directives.yaml_version.is_some() || directives.tag_prefixes.len() > 2;
    if has_explicit_directives {
        let st = source.save_state();
        let mut ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
        ts.skip_whitespace_and_comments()?;
        match ts.current() {
            Some(crate::parser::lexer::Token::DocumentStart) => {}
            Some(crate::parser::lexer::Token::DocumentEnd)
            | Some(crate::parser::lexer::Token::Eof)
            | None => {
                source.restore_state(st);
                return Err(helpers::parse_error(
                    source,
                    "Directive must be followed by a document",
                ));
            }
            _ => {}
        }
        source.restore_state(st);
    }
    Ok(())
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
    let mut saw_marker = false;
    let mut any_content = false;
    while source.more() {
        crate::utils::skip_whitespace_and_comments(source);
        // Always create a fresh DirectiveContext for each document
        let mut directives = crate::parser::directives::DirectiveContext::new();
        // Parse and apply any directives for this document
        let parsed_directives = parse_directives(source)?;
        // Merge parsed directives into the new context
        directives.yaml_version = parsed_directives.yaml_version;
        directives.tag_prefixes.extend(parsed_directives.tag_prefixes);
        check_explicit_directives(source, &directives)?;
        let marker_res = parse_document_markers(source, &directives);
        let marker_ok = marker_res.is_ok();
        if marker_ok {
            if saw_marker && !any_content {
                docs.push(Document(Vec::new()));
            }
            saw_marker = true;
            any_content = false;
        }
        let document = parse_document(source, 0, &directives);
        match document {
            Ok(doc) => {
                docs.push(doc);
                any_content = true;
            },
            Err(err) => return Err(err),
        }
        parse_document_end_marker(source, &directives)?;
        if !source.more() {
            break;
        }
    }
    if docs.is_empty() {
        docs.push(Document(Vec::new()));
    }
    if saw_marker && !any_content {
        docs.push(Document(Vec::new()));
    }
    #[cfg(feature = "debug-trace")]
    log::debug!("parse: end stream with {} document(s)", docs.len());
    Ok(Node::Documents(docs))
}
