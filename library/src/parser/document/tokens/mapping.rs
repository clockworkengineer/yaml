use crate::parser::document::error_builder::syntax_error;
use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::parser::directives::DirectiveContext;
use crate::parser::document::node_utils::force_key_to_string;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Parse a single key-value mapping pair (for sequence items)
#[allow(dead_code)]
pub fn parse_single_mapping_pair_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let (key, value) = parse_mapping_pair(stream, directives, 0, 0)?;
    Ok(Node::Mapping(vec![(key, value)]))
}

/// Parse a block mapping using tokens
///
/// Example:
/// ```yaml
/// key1: value1
/// key2: value2
/// !!str: tagged_key
/// ? complex_key
/// : complex_value
/// ```
///
/// Benefits of token-based approach:
/// - No complex lookahead for keys with decorators
/// - Clear token boundaries prevent infinite loops
/// - Natural handling of explicit keys (?)
#[allow(dead_code)]
pub fn parse_mapping_with_tokens(
    stream: &mut TokenStream,
    base_indent: usize,
    directives: &DirectiveContext,
    depth: usize,
) -> Result<Node, String> {
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_tokens: start parse_mapping_with_tokens");
    let mut stack: Vec<(usize, Vec<(Node, Node)>)> = Vec::new();
    stack.push((base_indent, Vec::new()));

    // Skip initial whitespace/newlines
    stream.skip_whitespace()?;

    loop {
        while matches!(
            stream.current(),
            Some(Token::Newline) | Some(Token::Comment(_))
        ) {
            stream.next()?;
        }

        // Before parsing a new key, check for dedent and unwind stack if needed
        let mut _dedented = false;
        loop {
            let current_indent = stack.last().map(|(lvl, _)| *lvl).unwrap_or(base_indent);
            let token_indent = match stream.current() {
                Some(Token::Indent(level)) => *level,
                _ => current_indent,
            };
            if token_indent < current_indent && stack.len() > 1 {
                // Pop stack frames until the current indent matches the token's indent
                let (_, closed_pairs) = stack.pop().unwrap();
                if let Some((_, parent_pairs)) = stack.last_mut() {
                    parent_pairs.push((Node::None, Node::Mapping(closed_pairs)));
                }
                _dedented = true;
            } else {
                break;
            }
        }

        let current_indent = stack.last().map(|(lvl, _)| *lvl).unwrap_or(base_indent);
        let token = stream.current().cloned();
        match token {
            Some(Token::Indent(level)) if level < current_indent => {
                println!(
                    "DEBUG: Dedent (level: {}, current_indent: {}, stack_len: {}) - closing current mapping",
                    level,
                    current_indent,
                    stack.len()
                );
                // Pop stack frames until the current indent matches the token's indent
                while stack.len() > 1 && stack.last().map(|(i, _)| *i).unwrap_or(0) > level {
                    let (_, closed_pairs) = stack.pop().unwrap();
                    println!(
                        "DEBUG: Dedent pop: stack now {:?}",
                        stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                    );
                    if let Some((_, parent_pairs)) = stack.last_mut() {
                        println!(
                            "DEBUG: Inserting dedented mapping into parent. Parent pairs before: {}",
                            parent_pairs.len()
                        );
                        parent_pairs.push((Node::None, Node::Mapping(closed_pairs)));
                        println!("DEBUG: Parent pairs after: {}", parent_pairs.len());
                    }
                }
                // After dedent, return to parent so the next key is parsed at the correct level
                let (_, pairs) = stack.last().unwrap();
                return Ok(Node::Mapping(pairs.clone()));
            }
            Some(Token::Eof) => {
                // At EOF: unwind the stack, closing all open mappings
                println!(
                    "DEBUG: EOF encountered, unwinding stack. stack_len={}",
                    stack.len()
                );
                while stack.len() > 1 {
                    let (top_indent, top_pairs) = stack.pop().unwrap();
                    println!(
                        "DEBUG: EOF unwind: closing mapping at indent {} with {} pairs",
                        top_indent,
                        top_pairs.len()
                    );
                    if let Some((_, parent_pairs)) = stack.last_mut() {
                        let mapping_node = Node::Mapping(top_pairs);
                        // Insert as value for last key in parent if possible
                        if let Some((_, last_value)) = parent_pairs.last_mut() {
                            println!(
                                "DEBUG: EOF unwind: inserting mapping_node as last_value in parent"
                            );
                            *last_value = mapping_node;
                        } else {
                            // If no key, push as orphan (should not happen in valid YAML)
                            println!("DEBUG: EOF unwind: pushing orphan mapping_node to parent");
                            parent_pairs.push((
                                force_key_to_string(Node::Str(
                                    "<unwound>".to_string(),
                                    QuoteType::Unquoted,
                                    BlockStyle::None,
                                )),
                                mapping_node,
                            ));
                        }
                        println!(
                            "DEBUG: Parent pairs after EOF unwind: {}",
                            parent_pairs.len()
                        );
                    }
                    println!(
                        "DEBUG: Stack after EOF unwind pop: {:?}",
                        stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                    );
                }
                let (_, pairs) = stack.pop().unwrap();
                println!(
                    "DEBUG: Final mapping pairs at EOF: {:?}",
                    pairs.iter().map(|(k, v)| (k, v)).collect::<Vec<_>>()
                );
                return Ok(Node::Mapping(pairs));
            }
            Some(Token::DocumentStart)
            | Some(Token::Dash)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd) => {
                // End of mapping
                let (_, pairs) = stack.pop().unwrap();
                return Ok(Node::Mapping(pairs));
            }
            Some(Token::DocumentEnd) => {
                // Document end marker - validate no content after it on same line
                // The lexer has already consumed "..." and positioned us right after it
                // Check for invalid content after ... on same line (before newline)
                loop {
                    match stream.source_mut().current() {
                        Some(' ') | Some('\t') => stream.source_mut().next(),
                        Some('#') => {
                            while let Some(c) = stream.source_mut().current() {
                                if c == '\n' || c == '\r' {
                                    break;
                                }
                                stream.source_mut().next();
                            }
                            break;
                        }
                        Some('\n') | Some('\r') | None => break,
                        Some(c) => {
                            return Err(syntax_error(
                                stream.source_mut(),
                                &format!("Invalid content '{}' after document end marker (...)", c)
                            ));
                        }
                    }
                }
                let (_, pairs) = stack.pop().unwrap();
                return Ok(Node::Mapping(pairs));
            }
            Some(Token::Newline) => {
                stream.next()?;
                continue;
            }
            Some(Token::Comment(_)) => {
                stream.next()?;
                continue;
            }
            Some(Token::Indent(level)) => {
                println!(
                    "TRACE: Token::Indent encountered: level={}, stack_len={}, stack={:?}",
                    level,
                    stack.len(),
                    stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                );
                let current_indent = stack.last().map(|(lvl, _)| *lvl).unwrap_or(base_indent);
                println!(
                    "DEBUG: INDENT token encountered: level={}, current_indent={}, stack_len={}",
                    level,
                    current_indent,
                    stack.len()
                );
                if level > current_indent {
                    // New nested mapping: push to stack
                    println!(
                        "DEBUG: Pushing new stack frame for nested mapping at indent {} (current_indent={}, stack_len={})",
                        level,
                        current_indent,
                        stack.len()
                    );
                    stack.push((level, Vec::new()));
                    println!(
                        "DEBUG: Stack after push: {:?}",
                        stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                    );
                    stream.next()?;
                    continue;
                } else if level < current_indent {
                    // Dedent: pop stack and insert completed mapping into parent
                    println!(
                        "DEBUG: Dedent detected: popping stack. level={}, current_indent={}, stack_len={}",
                        level,
                        current_indent,
                        stack.len()
                    );
                    // ...existing code for dedent...
                    stream.next()?;
                    continue;
                } else {
                    stream.next()?;
                    continue;
                }
            }
            _ => {}
        }
        println!(
            "TRACE: At top of loop, token={:?}, stack_len={}, stack={:?}",
            token,
            stack.len(),
            stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
        );
        match token {
            Some(Token::Newline) | Some(Token::Comment(_)) => {
                stream.next()?;
                continue;
            }
            Some(Token::Indent(level)) => {
                println!(
                    "TRACE: Token::Indent encountered: level={}, stack_len={}, stack={:?}",
                    level,
                    stack.len(),
                    stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                );
                let current_indent = stack.last().map(|(lvl, _)| *lvl).unwrap_or(base_indent);
                println!(
                    "DEBUG: INDENT token encountered: level={}, current_indent={}, stack_len={}",
                    level,
                    current_indent,
                    stack.len()
                );
                if level > current_indent {
                    // New nested mapping: push to stack
                    println!(
                        "DEBUG: Pushing new stack frame for nested mapping at indent {} (current_indent={}, stack_len={})",
                        level,
                        current_indent,
                        stack.len()
                    );
                    stack.push((level, Vec::new()));
                    println!(
                        "DEBUG: Stack after push: {:?}",
                        stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                    );
                    stream.next()?;
                    continue;
                } else if level < current_indent {
                    // Dedent: pop stack and insert completed mapping into parent
                    println!(
                        "DEBUG: Dedent detected: popping stack. level={}, current_indent={}, stack_len={}",
                        level,
                        current_indent,
                        stack.len()
                    );
                    let mut pop_count = 0;
                    while let Some((top_indent, top_pairs)) = stack.pop() {
                        println!(
                            "DEBUG: Popped stack frame: indent={}, pairs_len={}",
                            top_indent,
                            top_pairs.len()
                        );
                        pop_count += 1;
                        if top_indent == level {
                            println!(
                                "DEBUG: Reached matching indent, pushing frame back: indent={}, pairs_len={}",
                                top_indent,
                                top_pairs.len()
                            );
                            stack.push((top_indent, top_pairs));
                            break;
                        }
                        if let Some((parent_indent, parent_pairs)) = stack.last_mut() {
                            println!(
                                "DEBUG: Inserting completed mapping into parent (parent_indent={}, parent_pairs_len={})",
                                parent_indent,
                                parent_pairs.len()
                            );
                            let mapping_node = Node::Mapping(top_pairs);
                            parent_pairs.push((
                                force_key_to_string(Node::Str(
                                    "<nested>".to_string(),
                                    QuoteType::Unquoted,
                                    BlockStyle::None,
                                )),
                                mapping_node,
                            ));
                            println!("DEBUG: Parent pairs after insert: {}", parent_pairs.len());
                        } else {
                            println!("DEBUG: No parent found, pushing frame back as root");
                            stack.push((top_indent, top_pairs));
                            break;
                        }
                        if let Some((new_top_indent, _)) = stack.last() {
                            println!("DEBUG: Top of stack after pop: indent={}", new_top_indent);
                            if *new_top_indent <= level {
                                break;
                            }
                        } else {
                            println!("DEBUG: Stack is empty after pop");
                            break;
                        }
                    }
                    println!(
                        "DEBUG: Dedent pop loop exited after {} pops. Stack now: {:?}",
                        pop_count,
                        stack.iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>()
                    );
                    stream.next()?;
                    continue;
                } else {
                    stream.next()?;
                    continue;
                }
            }
            Some(Token::Eof)
            | Some(Token::DocumentEnd)
            | Some(Token::DocumentStart)
            | Some(Token::Dash)
            | Some(Token::FlowMappingEnd)
            | Some(Token::FlowSequenceEnd) => {
                let (_, pairs) = stack.pop().unwrap();
                return Ok(Node::Mapping(pairs));
            }
            _ => {
                let cur_indent = stack.last().map(|(lvl, _)| *lvl).unwrap_or(base_indent);
                let (key, value) = parse_mapping_pair(stream, directives, cur_indent, depth)?;
                let norm_key = force_key_to_string(key);
                // Debug: print key type before insertion
                #[cfg(debug_assertions)]
                println!("DEBUG: Inserting mapping key: {:?}", norm_key);
                // Patch: If the value is a Set with a single empty string, and the next token is an explicit key (?),
                // treat the following block as the value for this key (for !!set explicit block format)
                if let Node::Set(items) = &value {
                    let is_empty_str = items.len() == 1
                        && matches!(items[0], Node::Str(ref s, _, _) if s.is_empty());
                    let is_mapping_set = if let Some(Token::Plain(_)) = stream.current() {
                        true
                    } else {
                        false
                    };
                    if (is_empty_str && is_mapping_set)
                        || (is_empty_str && matches!(stream.current(), Some(Token::QuestionMark)))
                    {
                        let mapping_value =
                            parse_mapping_with_tokens(stream, cur_indent, directives, depth + 1)?;
                        if let Node::Mapping(ref pairs) = mapping_value {
                            let mut set_items = Vec::new();
                            for (k, v) in pairs {
                                if matches!(v, Node::None) {
                                    set_items.push(k.clone());
                                } else {
                                    // Not a valid set, return as mapping
                                    return Ok(Node::Mapping(pairs.clone()));
                                }
                            }
                            // FLATTEN: If the current value is a Set, merge its items
                            let mut all_items = Vec::new();
                            for item in items.iter() {
                                if !matches!(item, Node::Str(s, _, _) if s.is_empty()) {
                                    all_items.push(item.clone());
                                }
                            }
                            all_items.extend(set_items);
                            return Ok(Node::Set(all_items));
                        } else {
                            // Not a mapping, just return as-is
                            return Ok(mapping_value);
                        }
                    }
                }
                if let Some((_, _pairs)) = stack.last_mut() {
                    // Get stack info before mutable borrow
                    let stack_idx = stack.len().saturating_sub(1);
                    let (stack_indent, pairs_len) = if let Some((lvl, pairs)) = stack.get(stack_idx)
                    {
                        (*lvl, pairs.len())
                    } else {
                        (base_indent, 0)
                    };
                    println!(
                        "DEBUG: Inserting pair: key={:?}, value={:?} into stack at indent {} (pairs before: {})",
                        norm_key, value, stack_indent, pairs_len
                    );
                    if let Some((_, pairs)) = stack.last_mut() {
                        pairs.push((norm_key, value));
                        // After push, print only the new pairs length for this stack frame
                        println!(
                            "DEBUG: Stack at indent {} now has {} pairs",
                            stack_indent,
                            pairs.len()
                        );
                    }
                }
            }
        }
    }

    // unreachable: loop always returns on end condition
}

/// Parse a single key-value pair
#[allow(dead_code)]
fn parse_mapping_pair(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
    cur_indent: usize,
    depth: usize,
) -> Result<(Node, Node), String> {
    println!(
        "DEBUG: parse_mapping_pair: start, token = {:?}",
        stream.current()
    );
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: start at token = {:?}", stream.current());
    // Unify explicit and implicit key handling: always use token stream for key detection
    // If the next token is a question mark, it's an explicit key
    let mut explicit_key = false;
    if matches!(stream.current(), Some(Token::QuestionMark)) {
        stream.next()?;
        explicit_key = true;
    }

    // Handle decorators (tag/anchor) and parse the key value
    let key = {
        let mut parsed_key = if matches!(
            stream.current(),
            Some(Token::Tag(_)) | Some(Token::Anchor(_))
        ) {
            let decorators = stream.consume_decorators()?;
            println!(
                "DEBUG: parse_mapping_pair: after decorators, token = {:?}",
                stream.current()
            );
            if matches!(stream.current(), Some(Token::Colon)) {
                use crate::nodes::node::{BlockStyle, QuoteType};
                let mut node = Node::Str("".to_string(), QuoteType::Unquoted, BlockStyle::None);
                if let Some(tag) = decorators.tag {
                    node = Node::Tagged(Box::new(node), tag);
                }
                if let Some(anchor) = decorators.anchor {
                    node = Node::Anchored(Box::new(node), anchor);
                }
                node
            } else {
                parse_value_with_tokens(stream, directives, depth + 1)?
            }
        } else {
            parse_value_with_tokens(stream, directives, depth + 1)?
        };
        // If the key is an array, convert to string representation for mapping key (with double quotes)
        if let Node::Array(items) = &parsed_key {
            let mut s = String::from("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                match item {
                    Node::Str(val, _, _) => s.push_str(val),
                    Node::Number(n) => s.push_str(&format!("{:?}", n)),
                    Node::Boolean(b) => s.push_str(&format!("{}", b)),
                    _ => s.push_str(&format!("{:?}", item)),
                }
            }
            s.push(']');
            use crate::nodes::node::{BlockStyle, QuoteType};
            parsed_key = Node::Str(s, QuoteType::Double, BlockStyle::None);
        }
        parsed_key
    };
    println!(
        "DEBUG: parse_mapping_pair: after key, token = {:?}",
        stream.current()
    );
    // Allow newlines and whitespace after key before colon
    loop {
        match stream.current() {
            Some(Token::Newline) | Some(Token::Comment(_)) => {
                stream.next()?;
                continue;
            }
            _ => break,
        }
    }
    match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
        }
        _ if explicit_key => {
            // Explicit key may omit a value entirely (e.g., !!set with '? key').
            // YAML allows explicit keys without a following colon to indicate an
            // empty value. If the colon isn't found on the next non-trivia token,
            // treat the value as empty rather than error.
            // If the next token is a new key, document boundary, or EOF, treat as empty value
            // This is critical for !!set block format: ? item1\n? item2\n? item3
            // Always treat as Node::None if not followed by colon/value.
            println!(
                "DEBUG: parse_mapping_pair: after explicit key newline/whitespace, token = {:?}",
                stream.current()
            );
            match stream.current() {
                Some(Token::Plain(_))
                | Some(Token::Tag(_))
                | Some(Token::Anchor(_))
                | Some(Token::QuestionMark)
                | Some(Token::DocumentEnd)
                | Some(Token::DocumentStart)
                | Some(Token::Eof)
                | None => {
                    // Explicit key with no value: treat as Node::None
                    return Ok((key, Node::None));
                }
                // Also treat Indent as start of new mapping entry (Node::None)
                Some(Token::Indent(_)) => {
                    return Ok((key, Node::None));
                }
                _ => {
                    // Otherwise, always attempt to parse a value (let value parser handle)
                }
            }
            // If colon is not present, treat as empty value
            if !matches!(stream.current(), Some(Token::Colon)) {
                return Ok((key, Node::None));
            } else {
                stream.next()?;
            }
        }
        Some(Token::Eof) | None => {
            // Treat EOF or None as valid empty value
            return Ok((key, Node::None));
        }
        // If next token is a valid key, treat as empty value
        Some(Token::Plain(_))
        | Some(Token::Tag(_))
        | Some(Token::Anchor(_))
        | Some(Token::QuestionMark) => {
            return Ok((key, Node::None));
        }
        Some(Token::Dash) => {
            // Allow dash as value only if it follows a newline (handled above)
            return Ok((key, Node::None));
        }
        // Do not error on Indent or other tokens; let the value parser handle it
        _ => {
            // No error: let the value parser handle the next token
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: before value, token = {:?}", stream.current());
    // Parse the value - check for empty value BEFORE skipping whitespace
    let cur_token = stream.current().cloned();
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "mapping_pair: value parse branch, cur_token = {:?}",
        cur_token
    );
    let value = match cur_token {
        Some(Token::Newline) | None | Some(Token::Eof) => {
            // After colon and newline, skip any intervening comments/newlines,
            // then check for indent and dash for block sequence/mapping value.
            if matches!(stream.current(), Some(Token::Newline)) {
                stream.next()?;
            }
            // Allow comments and blank lines immediately after the colon
            loop {
                match stream.current() {
                    Some(Token::Newline) | Some(Token::Comment(_)) => {
                        stream.next()?;
                        continue;
                    }
                    _ => break,
                }
            }
            // Check for indent starting the nested value
            let indent_level = if let Some(Token::Indent(level)) = stream.current() {
                if *level > cur_indent {
                    let lvl = *level;
                    stream.next()?; // consume Indent
                    Some(lvl)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(level) = indent_level {
                // After indent, skip newlines/comments before checking for dash
                loop {
                    match stream.current() {
                        Some(Token::Newline) | Some(Token::Comment(_)) => {
                            stream.next()?;
                        }
                        _ => break,
                    }
                }
                println!(
                    "DEBUG: parse_mapping_pair: after value indent/newline, token = {:?}",
                    stream.current()
                );
                if matches!(stream.current(), Some(Token::Dash)) {
                    // Parse block sequence as value
                    use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: parsing sequence as value for key {:?}", key);
                    let ctx_seq = crate::parser::document::context::ParsingContext::new(level)
                        .child_block_context(
                            level,
                            crate::parser::document::context::CollectionType::BlockSequence,
                        );
                    let seq =
                        parse_sequence_with_tokens(stream, level, directives, &ctx_seq, depth + 1)?;
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: sequence node for key {:?}: {:?}", key, seq);
                    return Ok((key, seq));
                } else {
                    // Parse nested mapping as value
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: parsing mapping as value for key {:?}", key);
                    let map = parse_mapping_with_tokens(stream, level, directives, depth + 1)?;
                    #[cfg(feature = "debug-trace")]
                    log::debug!("mapping_pair: mapping node for key {:?}: {:?}", key, map);
                    return Ok((key, map));
                }
            }
            Node::None
        }
        Some(Token::Indent(level)) => {
            // Increased indentation: parse nested mapping or sequence as value
            stream.next()?; // consume Indent
            if matches!(stream.current(), Some(Token::Dash)) {
                use crate::parser::document::tokens::sequence::parse_sequence_with_tokens;
                let ctx_seq = crate::parser::document::context::ParsingContext::new(level)
                    .child_block_context(
                        level,
                        crate::parser::document::context::CollectionType::BlockSequence,
                    );
                parse_sequence_with_tokens(stream, level, directives, &ctx_seq, depth + 1)?
            } else {
                parse_mapping_with_tokens(stream, level, directives, depth + 1)?
            }
        }
        _ => {
            // Skip whitespace before value
            stream.skip_whitespace()?;
            // Parse the actual value
            let v = parse_value_with_tokens(stream, directives, depth + 1)?;
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: parsed value = {:?}", v);
            v
        }
    };
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_simple_mapping() {
        let yaml = b"key1: value1\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_mapping_with_empty_value() {
        let yaml = b"key1:\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_mapping_with_decorated_key() {
        let yaml = b"!!str: value\n&anchor: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged empty string
            // Second key should be anchored
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_fh7j_nested_mapping() {
        // FH7J has: "  !!null : a\n  b: !!str\n"
        let yaml = b"!!null: a\nb: !!str";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged null (empty)
            // Second value should be tagged empty string
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_keys_block_mapping() {
        // Explicit keys without values should produce Node::None values
        let yaml = b"? item1\n? item2\n? item3\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 3);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::None));
            assert!(matches!(pairs[2].1, Node::None));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_key_with_value() {
        // Explicit key followed by value on same line
        let yaml = b"? key1: value1\n? key2\n: value2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // key1 has value1; key2 should have value2
            assert!(matches!(pairs[0].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[0].1, Node::Str(ref s, _, _) if s == "value1"));
            assert!(matches!(pairs[1].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[1].1, Node::Str(ref s, _, _) if s == "value2"));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_complex_key_array() {
        // Complex explicit key (array) should normalize to string key
        let yaml = b"? [a, b, c]: 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Key should be a string representation of the array
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("a") && s.contains("b") && s.contains("c"))
            );
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_mapping() {
        let yaml = b"{}\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        // Inline empty mapping should parse via inline_tokens, but base parser should gracefully handle
        let node = crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
            &mut stream,
            &directives,
            0,
            false,
        )
        .unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_multiline_key_value_mapping() {
        // Multiline plain scalar key and value using block scalar-like lines
        let yaml = b"? |\n  multi\n  line\n: |\n  val\n  ue\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Keys/values produced by scalar parser should be strings (literal preserves newlines)
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("multi") && s.contains("line"))
            );
            assert!(
                matches!(pairs[0].1, Node::Str(ref s, _, _) if s.contains("val") && s.contains("ue"))
            );
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_value_on_same_line_and_next_line() {
        let yaml = b"key1: \nkey2:\n  - 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::Array(_)));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_decorated_empty_keys_tag_and_anchor() {
        // Decorated empty keys should produce empty-string keys wrapped by tag/anchor
        let yaml = b"!!str: one\n&root: two\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key is tagged empty string
            match &pairs[0].0 {
                Node::Tagged(inner, tag) => {
                    assert!(matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()));
                    assert!(tag.starts_with("!!") || tag.starts_with("!"));
                }
                other => panic!("Expected Tagged empty key, got {:?}", other),
            }
            // Second key is anchored empty string
            match &pairs[1].0 {
                Node::Anchored(inner, name) => {
                    assert_eq!(name, "root");
                    assert!(matches!(**inner, Node::Str(ref s, _, _) if s.is_empty()));
                }
                other => panic!("Expected Anchored empty key, got {:?}", other),
            }
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_keys_with_nested_sequence_values() {
        // Explicit keys followed by nested sequences
        let yaml = b"? key1\n: \n  - a\n  - b\n? key2\n: \n  - 1\n  - 2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives, 0).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::Array(ref v) if v.len() == 2));
            assert!(matches!(pairs[1].1, Node::Array(ref v) if v.len() == 2));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }
}
