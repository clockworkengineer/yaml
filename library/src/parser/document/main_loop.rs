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
    let mut document_nodes = Vec::new();
    while let Some(c) = source.current() {
        if is_document_marker(source, directives)? {
            break;
        }
        match c {
            '#' => {
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                stream.skip_whitespace_and_comments()?;
                continue;
            }
            '%' => {
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
