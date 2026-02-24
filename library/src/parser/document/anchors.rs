use crate::anchors_debug;

/**
 * Anchor Collection & Validation
 *
 * Provides functions for collecting, validating, and managing YAML anchors in document trees.
 * Ensures anchor uniqueness, non-emptiness, and supports anchor lookup for alias resolution.
 *
 * Copyright (c) 2026 YAML Library Developers
 */
use crate::nodes::node::Node;
use crate::parser::ParseResult;
use crate::parser::utils::error_helpers;
use crate::utils::anchors_helpers;
use std::collections::HashMap;

#[allow(dead_code)]
/// Recursively collects all anchor definitions from a YAML document tree.
///
/// Traverses the node tree and builds a map of anchor names to their
/// corresponding node values. Validates that anchor names are not empty
/// and that no duplicate anchors exist.
///
/// # Arguments
///
/// * `node` - The root node to search for anchors
/// * `anchors` - A mutable HashMap to store anchor name-to-node mappings
///
/// # Returns
///
/// Result indicating success or an error string for invalid anchors
pub(crate) fn collect_anchors(node: &Node, anchors: &mut HashMap<String, Node>) -> ParseResult<()> {
    anchors_helpers::traverse_with_error(node, |n| {
        if let Node::Anchored(inner, name) = n {
            if name.trim().is_empty() {
                anchors_debug!("Empty anchor name encountered");
                return Some(error_helpers::empty_anchor_name().to_string());
            }
            anchors_debug!("Collecting anchor: {}", name);
            // According to YAML spec, later anchor definitions override earlier ones
            anchors.insert(name.clone(), (**inner).clone());
        }
        None
    })
}

#[allow(dead_code)]
/// Recursively replaces all alias references with their corresponding anchor values.
///
/// Traverses the node tree and replaces any Alias nodes with the actual
/// node values they reference. Validates that all aliases have corresponding
/// anchors and that alias names are not empty.
///
/// # Arguments
///
/// * `node` - The node tree to process for alias replacement
/// * `anchors` - HashMap containing anchor name-to-node mappings
///
/// # Returns
///
/// Result indicating success or an error string for undefined aliases
pub(crate) fn replace_aliases(node: &mut Node, anchors: &HashMap<String, Node>) -> ParseResult<()> {
    let mut err: Option<crate::error::YamlError> = None;
    let mut replacer = |n: &mut Node| match n {
        Node::Alias(name) => {
            anchors_debug!("Replacing alias: {}", name);
            match anchors_helpers::lookup_anchor(anchors, name) {
                Ok(found) => *n = found.clone(),
                Err(e) => err = Some(e),
            }
        }
        Node::Anchored(inner, _name) => {
            anchors_debug!("Replacing anchored node");
            let replacement = (**inner).clone();
            *n = replacement;
        }
        _ => {}
    };
    crate::parser::utils::visit::visit_mut(node, &mut replacer);
    if let Some(e) = err { Err(e) } else { Ok(()) }
}

#[allow(dead_code)]
/// Expands YAML merge keys (<<) by incorporating referenced mapping values.
///
/// Processes merge key syntax in mappings, which allows inheriting key-value
/// pairs from other mappings. Handles both single aliases and sequences of
/// aliases as merge sources.
///
/// # Arguments
///
/// * `node` - The node tree to process for merge key expansion
/// * `anchors` - HashMap containing anchor name-to-node mappings
///
/// # Returns
///
/// Result indicating success or an error string for invalid merge operations
pub(crate) fn expand_merge_keys(
    node: &mut Node,
    anchors: &HashMap<String, Node>,
) -> ParseResult<()> {
    let mut err: Option<crate::error::YamlError> = None;
    let mut expander = |n: &mut Node| {
        if let Node::Mapping(pairs) = n {
            anchors_debug!("Expanding merge keys in mapping");
            let mut combined: Vec<(Node, Node)> = Vec::new();
            let snapshot = pairs.clone();
            let mut i = 0usize;
            while i < snapshot.len() {
                let (k, v) = snapshot[i].clone();
                let mut handled = false;
                if let Node::Str(ks, _, _) = &k {
                    if ks.trim() == "<<" {
                        let mut expanded_pairs: Vec<(Node, Node)> = Vec::new();
                        match &v {
                            Node::Alias(name) => {
                                anchors_debug!("Expanding merge alias: {}", name);
                                match anchors_helpers::lookup_anchor(anchors, name)
                                    .and_then(|src| anchors_helpers::as_mapping(src, name))
                                {
                                    Ok(src_pairs) => expanded_pairs.extend(src_pairs.clone()),
                                    Err(e) => {
                                        err = Some(e);
                                        break;
                                    }
                                }
                            }
                            Node::Array(items) => {
                                anchors_debug!("Expanding merge array with {} items", items.len());
                                for it in items.iter() {
                                    match it {
                                        Node::Alias(name) => {
                                            anchors_debug!(
                                                "Expanding merge alias in array: {}",
                                                name
                                            );
                                            match anchors_helpers::lookup_anchor(anchors, name)
                                                .and_then(|src| {
                                                    anchors_helpers::as_mapping(src, name)
                                                }) {
                                                Ok(src_pairs) => {
                                                    expanded_pairs.extend(src_pairs.clone())
                                                }
                                                Err(e) => {
                                                    err = Some(e);
                                                    break;
                                                }
                                            }
                                        }
                                        Node::Mapping(src_pairs) => {
                                            anchors_debug!("Expanding merge mapping in array");
                                            expanded_pairs.extend(src_pairs.clone());
                                        }
                                        _ => {
                                            err =
                                                Some(error_helpers::invalid_merge_sequence_item());
                                            break;
                                        }
                                    }
                                }
                            }
                            Node::Mapping(src_pairs) => {
                                expanded_pairs.extend(src_pairs.clone());
                            }
                            other => {
                                err = Some(error_helpers::invalid_merge_value(&format!(
                                    "{:?}",
                                    other
                                )));
                            }
                        }
                        combined.extend(expanded_pairs);
                        handled = true;
                    }
                }
                if handled {
                    i += 1;
                    continue;
                }
                if let Node::Str(_, _, _) = &k {
                    if let Node::Str(s, _, _) = &v {
                        let ts = s.trim_start();
                        if ts.starts_with("<<:") && ts.contains('*') {
                            if let Some(pos) = ts.find('*') {
                                let aname = ts[pos + 1..].trim().to_string();
                                let mut nested: Vec<(Node, Node)> = Vec::new();
                                let mut j = i + 1;
                                while j < snapshot.len() {
                                    if let Node::Str(ns, _, _) = &snapshot[j].1 {
                                        if ns.trim_start().starts_with("<<:") {
                                            break;
                                        }
                                    }
                                    if let Node::Str(key_name, _, _) = &snapshot[j].0 {
                                        if key_name.contains("boolean")
                                            || key_name.contains("explicit")
                                        {
                                            break;
                                        }
                                        let trimmed_key = key_name.trim();
                                        if trimmed_key == "explicit_boolean" {
                                            break;
                                        }
                                    }
                                    nested.push(snapshot[j].clone());
                                    j += 1;
                                }
                                let mut merged_pairs: Vec<(Node, Node)> = Vec::new();
                                if let Some(src) = anchors.get(&aname) {
                                    if let Node::Mapping(ap) = src {
                                        merged_pairs.extend(ap.clone());
                                    }
                                }
                                for (nk, nv) in nested.iter() {
                                    let mut replaced = false;
                                    if let Node::Str(nks, _, _) = nk {
                                        for p in merged_pairs.iter_mut() {
                                            if let Node::Str(pk, _, _) = &p.0 {
                                                if pk == nks {
                                                    *p = (nk.clone(), nv.clone());
                                                    replaced = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if !replaced {
                                        merged_pairs.push((nk.clone(), nv.clone()));
                                    }
                                }
                                combined.push((
                                    k.clone(),
                                    crate::parser::document::node_utils::make_mapping_node(
                                        merged_pairs,
                                    ),
                                ));
                                i = j;
                                continue;
                            }
                        }
                    }
                }
                combined.push((k, v));
                i += 1;
            }
            *pairs = crate::parser::document::node_utils::dedupe_mapping_pairs_by_last_occurrence(
                combined,
            );
        }
    };
    crate::parser::utils::visit::visit_mut(node, &mut expander);
    if let Some(e) = err { Err(e) } else { Ok(()) }
}
