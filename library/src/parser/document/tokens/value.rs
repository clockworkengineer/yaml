//! Token-based value parser (proof of concept)
//!
//! This demonstrates how the tokenization approach solves the decorator parsing
//! problem and eliminates infinite loops.

use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::directives::DirectiveContext;
use crate::parser::document::error_builder::{
    mapping_key_error_yaml, syntax_error,
};
use crate::error::{YamlError, ErrorKind};
const MAX_NESTING_DEPTH: usize = 128;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Try to coerce a value based on a tag
fn try_coerce_tag(tag: &str, node: Node) -> Option<Node> {
    match tag {
        "!!int:hex" => match node {
            Node::Str(s, _, _) => {
                if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    if let Ok(i) = i64::from_str_radix(stripped, 16) {
                        Some(Node::Number(Numeric::Integer(i)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!int:oct" => match node {
            Node::Str(s, _, _) => {
                if let Some(stripped) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
                    if let Ok(i) = i64::from_str_radix(stripped, 8) {
                        Some(Node::Number(Numeric::Integer(i)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!str" | "!str" | "tag:yaml.org,2002:str" => {
            let s = match node {
                Node::Str(s, _, _) => s,
                Node::Number(Numeric::Integer(i)) => i.to_string(),
                Node::Number(Numeric::Float(f)) => f.to_string(),
                Node::Boolean(b) => b.to_string(),
                Node::None => String::new(),
                _ => return None,
            };
            Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None))
        }
        "!!int" | "!int" | "tag:yaml.org,2002:int" => match node {
            Node::Number(Numeric::Integer(i)) => Some(Node::Number(Numeric::Integer(i))),
            Node::Str(s, _, _) => {
                // Accept both quoted and unquoted numbers, and also coerce floats that are integers
                if let Ok(i) = s.parse::<i64>() {
                    Some(Node::Number(Numeric::Integer(i)))
                } else if let Ok(f) = s.parse::<f64>() {
                    if f.fract() == 0.0 {
                        Some(Node::Number(Numeric::Integer(f as i64)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Node::Number(Numeric::Float(f)) => {
                if f.fract() == 0.0 {
                    Some(Node::Number(Numeric::Integer(f as i64)))
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!float" | "!float" | "tag:yaml.org,2002:float" => match node {
            Node::Number(Numeric::Float(f)) => Some(Node::Number(Numeric::Float(f))),
            Node::Number(Numeric::Integer(i)) => Some(Node::Number(Numeric::Float(i as f64))),
            Node::Str(s, _, _) => {
                if let Ok(f) = s.parse::<f64>() {
                    Some(Node::Number(Numeric::Float(f)))
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!bool" | "!bool" | "tag:yaml.org,2002:bool" => match node {
            Node::Boolean(b) => Some(Node::Boolean(b)),
            Node::Str(s, _, _) => {
                let sl = s.trim().to_ascii_lowercase();
                match sl.as_str() {
                    "true" | "yes" | "on" => Some(Node::Boolean(true)),
                    "false" | "no" | "off" => Some(Node::Boolean(false)),
                    _ => None,
                }
            }
            Node::Number(Numeric::Integer(i)) => Some(Node::Boolean(i != 0)),
            Node::Number(Numeric::Float(f)) => Some(Node::Boolean(f != 0.0)),
            _ => None,
        },
        "!!set" | "!set" | "tag:yaml.org,2002:set" => match node {
            // Convert mapping with null values to a set (DRY)
            Node::Mapping(pairs) => {
                if let Some(set_items) =
                    crate::parser::document::node_utils::pairs_to_set_items_if_all_none(&pairs)
                {
                    Some(Node::Set(set_items))
                } else {
                    None
                }
            }
            // Convert array to a set (remove duplicates)
            Node::Array(items) => {
                let mut unique_items = Vec::new();
                for item in items {
                    if !unique_items.contains(&item) {
                        unique_items.push(item);
                    }
                }
                Some(Node::Set(unique_items))
            }
            // FLATTEN: If already a set, flatten its items
            Node::Set(items) => {
                let mut flat = Vec::new();
                for item in items {
                    match item {
                        Node::Set(nested) => flat.extend(nested),
                        other => flat.push(other),
                    }
                }
                Some(Node::Set(flat))
            }
            // Single value becomes a set with one element
            _ => Some(Node::Set(vec![node])),
        },
        "!!omap" | "!omap" | "tag:yaml.org,2002:omap" => match node {
            // Ordered mapping - preserve as tagged array of single-key mappings
            Node::Array(items) => {
                // Validate that each item is a mapping with one key-value pair
                for item in &items {
                    match item {
                        Node::Mapping(pairs) if pairs.len() == 1 => {}
                        _ => return None, // Invalid omap format
                    }
                }
                Some(Node::Tagged(
                    Box::new(Node::Array(items)),
                    "tag:yaml.org,2002:omap".to_string(),
                ))
            }
            _ => None,
        },
        "!!pairs" | "!pairs" | "tag:yaml.org,2002:pairs" => match node {
            // Pairs - preserve as tagged array
            Node::Array(items) => Some(Node::Tagged(
                Box::new(Node::Array(items)),
                "tag:yaml.org,2002:pairs".to_string(),
            )),
            _ => None,
        },
        // Null coercion: map to Node::None
        "!!null" | "!null" | "tag:yaml.org,2002:null" => Some(Node::None),

        // Timestamp coercion: keep as plain string (tests expect string preservation)
        "!!timestamp" | "!timestamp" | "tag:yaml.org,2002:timestamp" => match node {
            Node::Str(s, _, _) => Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None)),
            // If numeric or boolean provided (unlikely), stringify
            Node::Number(Numeric::Integer(i)) => Some(Node::Str(
                i.to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            Node::Number(Numeric::Float(f)) => Some(Node::Str(
                f.to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            Node::Boolean(b) => Some(Node::Str(
                b.to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            Node::None => Some(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            _ => None,
        },

        _ => None,
    }
}

/// Parse a value using tokens (proof of concept)
///
/// This demonstrates how tokenization solves the decorator problem:
/// 1. Decorators are consumed upfront without complex lookahead
/// 2. No infinite loops - tokens have clear boundaries
/// 3. Empty values are naturally supported (EOF after decorator)
/// 4. Both tag+anchor and anchor+tag orderings work
pub fn parse_value_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    if depth > MAX_NESTING_DEPTH {
        return Err(YamlError::new(
            ErrorKind::ValidationError,
            "Nesting too deep: possible malicious or malformed YAML",
        ));
    }
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "value_tokens: ENTER parse_value_with_tokens (depth {}), current token: {:?}",
        depth,
        stream.current()
    );
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "value_tokens: parse_value_with_tokens (depth {}), current token: {:?}",
        depth,
        stream.current()
    );
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "value_tokens: start parse_value_with_tokens at token = {:?}",
        stream.current()
    );
    // Handle aliases first (they don't have content)
    if matches!(stream.current(), Some(Token::Alias(_))) {
        if let Some(Token::Alias(name)) = stream.current() {
            let alias = name.clone();
            stream.next()?;
            #[cfg(feature = "debug-trace")]
            log::debug!("value_tokens: parsed alias = {}", alias);
            return Ok(Node::Alias(alias));
        }
    }

    // Consume decorators using unified helper (NO INFINITE LOOPS!)
    let decorators = stream.consume_decorators()?;

    // If we have decorators, parse the content
    if decorators.tag.is_some() || decorators.anchor.is_some() {
        #[cfg(feature = "debug-trace")]
        log::debug!("value_tokens: decorators = {:?}", decorators);
        // Do NOT skip indentation/newlines here. Indentation signals nested
        // block structures (e.g., a mapping following a tag like !!set), and
        // consuming it would hide structure boundaries from the value parser.
        // Only skip comments; structural whitespace must be preserved.
        stream.skip_comments()?;

        // Special handling: if tag is !!seq, parse a sequence (block or flow)
        let tag_is_seq = decorators
            .tag
            .as_ref()
            .map(|t| {
                crate::parser::document::node_utils::resolved_is_seq(&directives.resolve_tag(t))
            })
            .unwrap_or(false);

        if tag_is_seq {
            // In flow context, a tagged sequence may be written as
            // `!!seq [ a, b ]` but split across lines, e.g. in EHF6:
            // `k: !!seq` then on the next indented line `[ a, !!str b ]`.
            // Inside a flow collection, indentation is just horizontal
            // whitespace, so we can safely skip Indent tokens here and
            // detect a following flow sequence start.
            if stream.current_flow_depth() > 0 {
                // Skip any indentation directly after the tag
                while matches!(stream.current(), Some(Token::Indent(_))) {
                    stream.next()?;
                }

                if matches!(stream.current(), Some(Token::FlowSequenceStart)) {
                    use crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens;
                    let seq = parse_inline_sequence_with_tokens(stream, directives, depth + 1)?;
                    let mut result =
                        Node::Tagged(Box::new(seq), "tag:yaml.org,2002:seq".to_string());
                    if let Some(anchor_name) = decorators.anchor {
                        result = Node::Anchored(Box::new(result), anchor_name);
                    }
                    return Ok(result);
                }
            }

            // Outside flow, an increased indentation after !!seq indicates
            // a block sequence value (e.g. `!!seq` followed by `- a`).
            if let Some(Token::Indent(level)) = stream.current() {
                use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
                let ctx_seq = crate::parser::document::context::ParsingContext::new(*level)
                    .child_block_context(
                        *level,
                        crate::parser::document::context::CollectionType::BlockSequence,
                    );
                let seq =
                    parse_sequence_with_tokens(stream, *level, 0, directives, &ctx_seq, depth + 1)?;
                let mut result = Node::Tagged(Box::new(seq), "tag:yaml.org,2002:seq".to_string());
                if let Some(anchor_name) = decorators.anchor {
                    result = Node::Anchored(Box::new(result), anchor_name);
                }
                return Ok(result);
            }
        }

        // NOW check if we're at EOF or end of structure (empty decorated value)
        // This includes:
        // - EOF/None: end of document
        // - Dash: next sequence item (e.g., "- !!str\n-" = empty tagged value)
        // - Colon: mapping key (e.g., "!!str :")
        // - Comma: next entry in a flow collection (e.g., "foo: !!str,")
        // - FlowMappingEnd/FlowSequenceEnd: end of flow collection
        // - DocumentStart/DocumentEnd: document boundary
        match stream.current() {
            Some(Token::Eof)
            | Some(Token::Dash)
            | Some(Token::Colon)
            | Some(Token::Comma)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd)
            | Some(Token::DocumentStart)
            | Some(Token::DocumentEnd)
            | None => {
                // Decorator with no content - empty value
                let mut result = Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None);

                if let Some(tag_raw) = decorators.tag {
                    let resolved = directives.resolve_tag(&tag_raw);
                    if let Some(coerced) = try_coerce_tag(&resolved, result.clone()) {
                        result = coerced;
                    } else {
                        // Preserve the original tag text when not coercing
                        result = Node::Tagged(Box::new(result), tag_raw);
                    }
                }

                if let Some(anchor_name) = decorators.anchor {
                    result = Node::Anchored(Box::new(result), anchor_name);
                }

                #[cfg(feature = "debug-trace")]
                log::debug!("value_tokens: empty decorated value -> {:?}", result);
                return Ok(result);
            }
            _ => {
                // There's content after decorators, continue to parse it
            }
        }

        // Special handling: if tag is !!set and next token is FlowMappingStart, parse as set
        let tag_is_set = decorators
            .tag
            .as_ref()
            .map(|t| {
                crate::parser::document::node_utils::resolved_is_set(&directives.resolve_tag(t))
            })
            .unwrap_or(false);

        let mut result = if tag_is_set {
            if matches!(stream.current(), Some(Token::FlowMappingStart)) {
                // Use set mode for inline mapping
                crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
                    stream,
                    directives,
                    depth + 1,
                    true,
                )?
            } else {
                parse_value_content(stream, directives, depth + 1)?
            }
        } else {
            parse_value_content(stream, directives, depth + 1)?
        };

        if let Some(tag_raw) = decorators.tag {
            let tag_resolved = directives.resolve_tag(&tag_raw);
            if tag_resolved == "!!str"
                || tag_resolved == "!str"
                || tag_resolved == "tag:yaml.org,2002:str"
            {
                // For block scalars, preserve block style; otherwise, coerce to plain string
                match &result {
                    Node::Str(s, q, style)
                        if *style == BlockStyle::Literal || *style == BlockStyle::Folded =>
                    {
                        result = Node::Str(s.clone(), q.clone(), style.clone());
                    }
                    Node::Str(s, _, _) => {
                        result = Node::Str(s.clone(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::Number(Numeric::Integer(i)) => {
                        result = Node::Str(i.to_string(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::Number(Numeric::Float(f)) => {
                        result = Node::Str(f.to_string(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::Boolean(b) => {
                        result = Node::Str(b.to_string(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    Node::None => {
                        result = Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None);
                    }
                    _ => {
                        result = Node::Str(
                            format!("{:?}", result),
                            QuoteType::Unquoted,
                            BlockStyle::None,
                        );
                    }
                }
            } else if crate::parser::document::node_utils::resolved_is_seq(&tag_resolved) {
                // Always wrap as Tagged with canonical tag for sequences
                match &result {
                    Node::Array(items) => {
                        result = Node::Tagged(
                            Box::new(Node::Array(items.clone())),
                            "tag:yaml.org,2002:seq".to_string(),
                        );
                    }
                    _ => {
                        // If not an array, still wrap whatever is there
                        result =
                            Node::Tagged(Box::new(result), "tag:yaml.org,2002:seq".to_string());
                    }
                }
            } else if crate::parser::document::node_utils::resolved_is_map(&tag_resolved) {
                // Always wrap as Tagged with canonical tag for mappings
                match &result {
                    Node::Mapping(pairs) => {
                        result = Node::Tagged(
                            Box::new(Node::Mapping(pairs.clone())),
                            "tag:yaml.org,2002:map".to_string(),
                        );
                    }
                    _ => {
                        result =
                            Node::Tagged(Box::new(result), "tag:yaml.org,2002:map".to_string());
                    }
                }
            } else if let Some(coerced) = try_coerce_tag(&tag_resolved, result.clone()) {
                result = coerced;
            } else {
                // Preserve the original tag text when not coercing
                result = Node::Tagged(Box::new(result), tag_raw);
            }
        }

        if let Some(anchor_name) = decorators.anchor {
            // SR86 / SU74: YAML 1.2 does not allow anchors to be applied
            // directly to alias nodes (e.g., "&b *a" or "&b *alias : v").
            // If the decorated value resolved to an alias, treat this as a
            // structural error rather than accepting an anchored alias.
            if matches!(result, Node::Alias(_)) {
                return Err(mapping_key_error_yaml(
                    stream.source_mut(),
                    "Invalid anchored alias: anchors cannot be applied to alias nodes",
                ));
            }
            if matches!(result, Node::Anchored(_, _)) {
                return Err(mapping_key_error_yaml(
                    stream.source_mut(),
                    "A node cannot have multiple anchors",
                ));
            }
            result = Node::Anchored(Box::new(result), anchor_name);
        }

        #[cfg(feature = "debug-trace")]
        log::debug!("value_tokens: decorated value -> {:?}", result);
        return Ok(result);
    }

    // No decorators - parse plain value
    let node = parse_value_content(stream, directives, depth + 1);
    #[cfg(feature = "debug-trace")]
    if let Ok(ref n) = node {
        log::debug!("value_tokens: plain value -> {:?}", n);
    }
    node
}

/// Parse value content (the actual value after decorators)
fn parse_value_content(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "value_tokens: parse_value_content at token = {:?}",
        stream.current()
    );
    // Skip comments before parsing value (DRY)
    stream.skip_comments()?;
    match stream.current() {
        // Tolerate stray commas in block contexts: consume and treat as empty value
        Some(Token::Comma) => {
            stream.next()?;
            stream.skip_trivia()?;
            return Ok(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ));
        }
        Some(Token::FlowMappingStart) => {
            use crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens;
            parse_inline_mapping_with_tokens(stream, directives, depth + 1, false)
        }
        Some(Token::FlowSequenceStart) => {
            use crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens;
            parse_inline_sequence_with_tokens(stream, directives, depth + 1)
        }
        Some(Token::SingleQuoted(_)) | Some(Token::DoubleQuoted(_)) | Some(Token::Plain(_)) => {
            crate::parser::document::scalar::parse_scalar_with_tokens(stream, directives, 0)
        }
        Some(Token::Dash) => {
            use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
            let ctx_seq = crate::parser::document::context::ParsingContext::new(0)
                .child_block_context(
                    0,
                    crate::parser::document::context::CollectionType::BlockSequence,
                );
            parse_sequence_with_tokens(stream, 0, 0, directives, &ctx_seq, depth + 1)
        }
        Some(Token::Indent(level)) => {
            // Indented value: parse nested mapping
            use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
            parse_mapping_with_tokens(stream, *level, directives, depth + 1)
        }
        Some(Token::Newline) => {
            // Handle newline before an indented block value. If there's an increased indent
            // after the newline, parse a nested mapping or sequence as the value.
            stream.next()?; // consume Newline
            // Check for indentation increase
            if let Some(Token::Indent(level)) = stream.current() {
                if *level > 0 {
                    let _lvl = *level;
                    stream.next()?; // consume Indent
                    // Skip subsequent newlines/comments
                    stream.skip_newlines_and_comments()?;
                    // Decide between sequence or mapping based on next token
                    if matches!(stream.current(), Some(Token::Dash)) {
                        use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
                        let ctx_seq = crate::parser::document::context::ParsingContext::new(_lvl)
                            .child_block_context(
                                _lvl,
                                crate::parser::document::context::CollectionType::BlockSequence,
                            );
                        return parse_sequence_with_tokens(
                            stream,
                            _lvl,
                            0,
                            directives,
                            &ctx_seq,
                            depth + 1,
                        );
                    } else {
                        use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
                        return parse_mapping_with_tokens(stream, _lvl, directives, depth + 1);
                    }
                }
            }
            // No indentation increase; treat as empty string value
            Ok(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ))
        }
        Some(Token::Eof) | None => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(Token::Alias(name)) => {
            let alias_name = name.clone();
            stream.next()?;
            Ok(Node::Alias(alias_name))
        }
        Some(Token::Colon) => Ok(Node::Str(
            String::new(),
            QuoteType::Unquoted,
            BlockStyle::None,
        )),
        Some(Token::QuestionMark) => {
            // Explicit key marker - parse as mapping with explicit keys
            use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
            // Parse mapping at indent 0 (explicit keys can appear at any indent)
            parse_mapping_with_tokens(stream, 0, directives, depth + 1)
        }
        Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd) => {
            // Flow collection end with no value = implicit null/empty
            // e.g., {key:} or [item,]
            Ok(Node::None)
        }
        Some(token) => {
            let token_str = format!("Unexpected token in value: {:?}", token);
            #[cfg(feature = "debug-trace")]
            log::debug!("value_tokens: error -> {}", token_str);
            Err(syntax_error(stream.source_mut(), &token_str))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_tag_on_empty_value() {
        // This is the FH7J pattern that caused infinite loops!
        let mut source = Buffer::new(b"!!str");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as empty string
        assert!(matches!(result, Node::Str(s, _, _) if s.is_empty()));
    }

    #[test]
    fn test_anchor_on_empty_value() {
        // This is the PW8X pattern that caused infinite loops!
        let mut source = Buffer::new(b"&anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as anchored empty string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_both_decorators_on_empty() {
        let mut source = Buffer::new(b"!!str &anchor");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as anchored empty string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s.is_empty()));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_tag_with_plain_value() {
        let mut source = Buffer::new(b"!!str hello");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        stream.next().unwrap(); // Initialize
        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as string "hello"
        assert!(matches!(result, Node::Str(s, _, _) if s == "hello"));
    }

    #[test]
    fn test_anchor_with_quoted_value() {
        let mut source = Buffer::new(b"&anchor 'hello'");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as anchored string
        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "anchor");
                assert!(matches!(*inner, Node::Str(s, _, _) if s == "hello"));
            }
            _ => panic!("Expected anchored node"),
        }
    }

    #[test]
    fn test_alias() {
        let mut source = Buffer::new(b"*myalias");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // Should parse as alias
        assert!(matches!(result, Node::Alias(name) if name == "myalias"));
    }

    #[test]
    fn test_error_on_multiple_anchors_for_single_node() {
        // Two anchors adjacent should error (single-anchor per node)
        let mut source = Buffer::new(b"&a &b 123");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let err = parse_value_with_tokens(&mut stream, &directives, 0).unwrap_err();
        let err_str = err.to_string().to_ascii_lowercase();
        assert!(
            err_str.contains("duplicate anchor")
                || err_str.contains("multiple anchors")
        );
    }

    #[test]
    fn test_tag_followed_by_indented_mapping() {
        // Decorator then indented block value should parse nested mapping
        let mut source = Buffer::new(b"!!set\n  a: null\n  b: null\n");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        // !!set should coerce mapping/nulls into a Set
        match result {
            Node::Set(items) => {
                assert_eq!(items.len(), 2);
            }
            other => panic!("Expected Set, got {:?}", other),
        }
    }

    #[test]
    fn test_anchor_followed_by_indented_mapping() {
        // Anchor then indented block value should wrap nested mapping in Anchored
        let mut source = Buffer::new(b"&root\n  key: value\n");
        let directives = DirectiveContext::default();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_value_with_tokens(&mut stream, &directives, 0).unwrap();

        match result {
            Node::Anchored(inner, name) => {
                assert_eq!(name, "root");
                assert!(matches!(*inner, Node::Mapping(_)));
            }
            _ => panic!("Expected Anchored mapping"),
        }
    }
}
