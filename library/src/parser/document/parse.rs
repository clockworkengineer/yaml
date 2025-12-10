use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::parse_directives;

use crate::parser::document::document_parser::parse_document;
use crate::parser::document::helpers;

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
            let mut ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
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
                    return Err(helpers::parse_error(
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
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
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

            // After ---, only whitespace/comments, block scalar indicators (|, >), or tags (!) are allowed until end of line
            // crate::utils::skip_whitespace_and_comments(source);
            if let Some(c) = source.current() {
                // Allow: newline, carriage return, comment, block scalar indicators, tags
                if c != '\n'
                    && c != '\r'
                    && c != '#'
                    && c != '>'
                    && c != '|'
                    && c != '!'
                    && c != '-'
                {
                    // Any other content (including mapping keys/values) is invalid on the marker line
                    return Err(helpers::parse_error(
                        source,
                        "YAML 1.2: Document start marker (---) must be on its own line, except for comments, block scalar indicators (|, >), or tags (!). No mapping keys or values allowed on the same line as ---.",
                    ));
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
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
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
                if c != '\n' && c != '\r' && c != '#' && c != '%' && c != '-' {
                    // There's non-whitespace, non-comment, non-directive content after ...
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
