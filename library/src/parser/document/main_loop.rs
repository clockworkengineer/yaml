use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::contents::parse_document_contents;
use crate::parser::document::helpers::node_is_blank;

/// Checks if the current position is at a document marker (--- or ...).
fn is_document_marker(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> Result<bool, String> {
    let st = source.save_state();
    let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
    let res = matches!(
        ts.current(),
        Some(crate::parser::lexer::Token::DocumentStart | crate::parser::lexer::Token::DocumentEnd)
    );
    source.restore_state(st);
    Ok(res)
}

/// Normalizes document nodes, handling mapping/array edge cases.
fn normalize_document_nodes(document_nodes: &[Node]) -> Vec<Node> {
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
    normalized_nodes
}

/// Main loop for parsing a single YAML document.
fn parse_document_main_loop(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &DirectiveContext,
) -> Result<Vec<Node>, String> {
    use crate::parser::document::context::ParsingContext;
    let mut document_nodes = Vec::new();
    let root_ctx = ParsingContext::new(indent_level);
    while let Some(c) = source.current() {
        if is_document_marker(source, directives)? {
            break;
        }
        match c {
            '#' => {
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;

                // Consume consecutive comment tokens starting at this position
                while matches!(stream.current(), Some(crate::parser::lexer::Token::Comment(_))) {
                    stream.next()?;
                }

                // If the comment is followed by a newline, inspect the indentation
                // of the next line to catch patterns like 8XDJ where an indented
                // scalar appears after a top-level comment with no enclosing
                // block structure. In such cases, the official YAML test suite
                // expects an error rather than silently treating the indented
                // scalar as a separate top-level document node.
                if let Some(crate::parser::lexer::Token::Newline) = stream.current() {
                    // Move to the first token on the following line
                    stream.next()?;
                    match stream.current().cloned() {
                        Some(crate::parser::lexer::Token::Indent(level)) if level > indent_level => {
                            // Consume the Indent token, then skip any additional
                            // newlines or comments to see what real content follows.
                            stream.next()?;
                            stream.skip_newlines_and_comments()?;
                            match stream.current() {
                                // If the next significant token is a value-like token
                                // (plain scalar or the start of a flow collection),
                                // report this as invalid indented content after a
                                // top-level comment.
                                Some(crate::parser::lexer::Token::Plain(_))
                                | Some(crate::parser::lexer::Token::SingleQuoted(_))
                                | Some(crate::parser::lexer::Token::DoubleQuoted(_))
                                | Some(crate::parser::lexer::Token::FlowMappingStart)
                                | Some(crate::parser::lexer::Token::FlowSequenceStart) => {
                                    return Err(crate::parser::document::helpers::parse_error_token(
                                        &stream,
                                        "Unexpected indented content after top-level comment.",
                                    ));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                // Preserve existing behavior of skipping over the comment and
                // any associated trivia before continuing the main loop.
                stream.skip_trivia()?;
                continue;
            }
            '%' => {
                break;
            }
            _ => {
                let node = parse_document_contents(source, indent_level, directives, &root_ctx)?;
                if !node_is_blank(&node) {
                    document_nodes.push(node);
                }
            }
        }
    }
    Ok(document_nodes)
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
    let document_nodes = parse_document_main_loop(source, indent_level, directives)?;
    let normalized_nodes = normalize_document_nodes(&document_nodes);
    let doc_node = Document(normalized_nodes);
    #[cfg(feature = "debug-trace")]
    {
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
