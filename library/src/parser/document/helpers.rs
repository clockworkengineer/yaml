use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::utils::*;

pub(crate) fn parse_error(source: &mut dyn ISource, msg: &str) -> String {
    let current = match source.current() {
        Some(c) => c.to_string(),
        None => STR_EOF.to_string(),
    };
    format!(
        "{} (current: '{}', indent: {})",
        msg,
        current,
        source.get_current_indent_level()
    )
}

pub(crate) fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if source.is_whitespace(c) {
            source.next();
        } else {
            break;
        }
    }
}

pub(crate) fn node_is_blank(node: &Node) -> bool {
    match node {
        Node::None => true,
        Node::Array(items) => items.is_empty(),
        Node::Mapping(_pairs) => false,
        Document(nodes) => nodes.iter().all(node_is_blank),
        Node::Str(s, _, _) => s.is_empty(),
        Node::Comment(_) => true,
        Node::Anchored(inner, _name) => node_is_blank(inner),
        Node::Alias(_name) => false,
        _ => false,
    }
}

pub(crate) fn parse_quoted_scalar(source: &mut dyn ISource) -> Result<String, String> {
    let quote = match source.current() {
        Some(c) if c == CHAR_SINGLE_QUOTE || c == CHAR_DOUBLE_QUOTE => c,
        Some(other) => {
            let msg = ERR_EXPECT_QUOTE_FORMAT.replace("{}", &other.to_string());
            return Err(parse_error(source, &msg).to_string());
        }
        None => return Err(parse_error(source, ERR_UNEXPECTED_EOF_EXPECTING_QUOTE)),
    };
    let mut out = String::new();
    out.push(quote);
    source.next();

    let mut prev_was_backslash = false;
    loop {
        match source.current() {
            Some(c) => {
                out.push(c);
                source.next();

                if c == quote {
                    if quote == CHAR_SINGLE_QUOTE {
                        if source.current() == Some(CHAR_SINGLE_QUOTE) {
                            out.push(CHAR_SINGLE_QUOTE);
                            source.next();
                            continue;
                        } else {
                            break;
                        }
                    } else if prev_was_backslash {
                        prev_was_backslash = false;
                        continue;
                    } else {
                        break;
                    }
                }

                if quote == CHAR_DOUBLE_QUOTE {
                    if c == CHAR_BACKSLASH {
                        prev_was_backslash = !prev_was_backslash;
                    } else {
                        prev_was_backslash = false;
                    }
                }
            }
            None => {
                return Err(parse_error(source, ERR_UNTERMINATED_QUOTED_FLOW));
            }
        }
    }

    Ok(out)
}

pub(crate) fn peek_ahead_for_document_start_end(source: &mut dyn ISource, c: char) -> bool {
    if source.current() != Some(c) {
        return false;
    }
    let state = source.save_state();
    source.next();
    if source.current() != Some(c) {
        source.restore_state(state);
        return false;
    }
    source.next();
    if source.current() != Some(c) {
        source.restore_state(state);
        return false;
    }
    source.restore_state(state);
    true
}

pub(crate) fn peek_ahead_for_mapping_key(source: &mut dyn ISource) -> bool {
    let mut found = false;
    let state = source.save_state();
    while let Some(c) = source.current() {
        match c {
            CHAR_COLON => {
                found = true;
                break;
            }
            CHAR_NEWLINE => {
                break;
            }
            _ => {
                if source.more() {
                    source.next();
                }
            }
        }
    }
    source.restore_state(state);
    found
}

pub(crate) fn parse_mapping_key(source: &mut dyn ISource) -> Result<(Node, bool), String> {
    let raw = collect_until(source, |c| c == CHAR_COLON || c == CHAR_NEWLINE);
    let mut newline = false;
    source.next();
    skip_whitespace(source);
    if let Some(c) = source.current() {
        if c == CHAR_HASH {
            consume_inline_comment_and_newline(source);
            newline = true;
        } else {
            newline = c == CHAR_NEWLINE;
            if newline {
                source.next();
                skip_whitespace(source);
            }
        }
    }

    match raw.trim() {
        v if v.starts_with(CHAR_HASH) => Ok((
            Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None),
            newline,
        )),
        v => Ok((crate::parser::document::scalar::parse_scalar(v), newline)),
    }
}

pub(crate) fn parse_comment(source: &mut dyn ISource) -> String {
    source.next();
    read_line_trimmed_into_string(source)
}

pub(crate) fn node_to_inline_string(node: &Node) -> String {
    crate::utils::node_to_inline_string(node)
}
