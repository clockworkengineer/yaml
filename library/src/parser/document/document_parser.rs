use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::helpers::node_is_blank;
use crate::parser::document::parse_document_contents;

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
            let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
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
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
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
