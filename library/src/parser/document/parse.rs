use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::parse_directives;

use crate::parser::document::main_loop::parse_document;
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
        // After --- marker, only allow whitespace, comments, block scalar indicators, or tags until end of line
        // Use token stream to check for forbidden tokens before newline
        let st = source.save_state();
        if let Ok(mut ts) = crate::parser::token_stream::TokenStream::new(source, directives, false)
        {
            // Skip whitespace tokens after ---
            loop {
                match ts.current() {
                    Some(crate::parser::lexer::Token::Indent(_)) => {
                        ts.next().ok();
                    }
                    _ => break,
                }
            }
            // Now check for comment or other allowed/forbidden tokens
            match ts.current() {
                Some(crate::parser::lexer::Token::Newline)
                | Some(crate::parser::lexer::Token::Eof) => {}
                Some(crate::parser::lexer::Token::Comment(_)) => {} // allow comment after ---
                Some(crate::parser::lexer::Token::Tag(_)) => {
                    ts.next().ok();
                }
                Some(crate::parser::lexer::Token::Plain(s)) => {
                    // Accept block scalar indicator (| or >) with optional indentation/chomping (e.g., |0, |1, >2, |+)
                    let trimmed = s.trim();
                    if trimmed.starts_with('|') || trimmed.starts_with('>') {
                        // Accept, let downstream handle block scalar validation
                    } else {
                        return Err(helpers::parse_error_token(
                            &ts,
                            "YAML 1.2: Document start marker (---) must be on its own line. No mapping keys or values allowed on the same line as ---.",
                        ));
                    }
                }
                Some(crate::parser::lexer::Token::Colon) => {
                    return Err(helpers::parse_error_token(
                        &ts,
                        "YAML 1.2: Document start marker (---) must be on its own line. No mapping keys or values allowed on the same line as ---.",
                    ));
                }
                Some(_) => {
                    return Err(helpers::parse_error_token(
                        &ts,
                        "YAML 1.2: Document start marker (---) must be on its own line. No mapping keys or values allowed on the same line as ---.",
                    ));
                }
                None => {}
            }
        }
        source.restore_state(st);
        // Move to next line if needed
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
        // Validate only inline content after '...' up to end-of-line
        loop {
            match source.current() {
                Some(' ') | Some('\t') => {
                    source.next();
                }
                Some('#') => {
                    // Inline comment: consume until end of line
                    while let Some(c) = source.current() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        source.next();
                    }
                }
                Some('\n') | Some('\r') | None => break,
                Some(_) => {
                    let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                    return Err(helpers::parse_error_token(
                        &ts,
                        "Invalid content after document end marker (...)",
                    ));
                }
            }
        }
        // Consume one optional Windows or Unix newline if present
        if source.current() == Some('\r') {
            source.next();
            if source.current() == Some('\n') {
                source.next();
            }
        } else if source.current() == Some('\n') {
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
                let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                return Err(helpers::parse_error_token(
                    &ts,
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
    let mut doc_count = 0;
    // Print the input for debug
    #[cfg(debug_assertions)]
    eprintln!("DEBUG: Starting parse() with YAML input");
    while source.more() {
        crate::utils::skip_whitespace_and_comments(source);
        // Always create a fresh DirectiveContext for each document
        let mut directives = crate::parser::directives::DirectiveContext::new();
        // Parse and apply any directives for this document
        let parsed_directives = parse_directives(source)?;
        // Merge parsed directives into the new context
        directives.yaml_version = parsed_directives.yaml_version;
        directives
            .tag_prefixes
            .extend(parsed_directives.tag_prefixes);
        check_explicit_directives(source, &directives)?;

        // Detect if there is a document start marker (---) at the current position
        let has_document_start = {
            let st = source.save_state();
            let ts = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
            let res = matches!(ts.current(), Some(crate::parser::lexer::Token::DocumentStart));
            source.restore_state(st);
            res
        };

        let marker_res = parse_document_markers(source, &directives);
        eprintln!("DEBUG: parse_document_markers result: {:?}", marker_res);
        if let Err(err) = marker_res {
            return Err(err);
        }
        if has_document_start {
            if saw_marker && !any_content {
                // Consecutive '---' with no content in between - treat as empty document
                // The suite merges/ignores these, so do not push an extra empty doc here.
            }
            saw_marker = true;
            // Consume any additional consecutive document start markers on following lines
            loop {
                let st2 = source.save_state();
                let ts2 = crate::parser::token_stream::TokenStream::new(source, &directives, false)?;
                let next_is_start = matches!(ts2.current(), Some(crate::parser::lexer::Token::DocumentStart));
                source.restore_state(st2);
                if next_is_start {
                    parse_document_markers(source, &directives)?;
                    // continue the loop to collapse runs of '---'
                    continue;
                }
                break;
            }
        }
        // Ensure we start the document after any trailing blank lines/comments following markers
        crate::utils::skip_whitespace_and_comments(source);
        let document = parse_document(source, 0, &directives);
        match document {
            Ok(Document(nodes)) => {
                // If parse_document returned multiple top-level nodes, it may indicate
                // multiple documents without proper loop splitting (e.g., stray markers).
                // However, sometimes incomplete nodes (e.g., a mapping with None value)
                // can be emitted transiently for a single logical document (e.g., block scalars).
                // Only split into multiple documents when ALL nodes are complete top-level values.
                let all_complete = nodes.iter().all(|n| match n {
                    Node::Mapping(pairs) => pairs.iter().all(|(_, v)| !matches!(v, Node::None)),
                    Node::None => false,
                    _ => true,
                });
                if nodes.len() > 1 && all_complete {
                    // If multiple nodes are returned but all except one are empty structures,
                    // keep only the non-empty one to avoid emitting spurious empty documents
                    let mut non_empty: Vec<&Node> = nodes
                        .iter()
                        .filter(|n| match n {
                            Node::Mapping(p) => !p.is_empty(),
                            Node::Array(a) => !a.is_empty(),
                            Node::None => false,
                            _ => true,
                        })
                        .collect();
                    if non_empty.len() == 1 {
                        doc_count += 1;
                        let doc = Document(vec![(*non_empty.pop().unwrap()).clone()]);
                        eprintln!("DEBUG: Parsed document #{}: {:#?}", doc_count, doc);
                        docs.push(doc);
                    } else {
                        for node in nodes {
                            doc_count += 1;
                            let single_doc = Document(vec![node.clone()]);
                            eprintln!("DEBUG: Parsed document #{}: {:#?}", doc_count, single_doc);
                            docs.push(single_doc);
                        }
                    }
                } else {
                    doc_count += 1;
                    let doc = Document(nodes);
                    eprintln!("DEBUG: Parsed document #{}: {:#?}", doc_count, doc);
                    docs.push(doc);
                }
                any_content = true;
            }
            Ok(doc) => {
                doc_count += 1;
                eprintln!("DEBUG: Parsed document #{}: {:#?}", doc_count, doc);
                docs.push(doc);
                any_content = true;
            }
            Err(err) => {
                eprintln!("DEBUG: Error parsing document #{}: {}", doc_count + 1, err);
                return Err(err);
            }
        }
        parse_document_end_marker(source, &directives)?;
        if !source.more() {
            break;
        }
    }
    eprintln!("DEBUG: Total documents parsed: {}", docs.len());
    if docs.is_empty() {
        docs.push(Document(Vec::new()));
    }
    if saw_marker && !any_content {
        docs.push(Document(Vec::new()));
    }
    eprintln!("DEBUG: Final docs vector: {:#?}", docs);
    #[cfg(feature = "debug-trace")]
    log::debug!("parse: end stream with {} document(s)", docs.len());
    Ok(Node::Documents(docs))
}
