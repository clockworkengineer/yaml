//! Helper functions for parsing block scalars (literal | and folded >)

use crate::constants::*;
use crate::io::traits::ISource;
use crate::nodes::node::{BlockStyle, Node, QuoteType};

/// Parses a block scalar (literal | or folded >)
/// 
/// Returns a tuple of (content_string, is_folded)
pub(crate) fn parse_block_scalar(
    source: &mut dyn ISource,
    is_folded: bool,
) -> Result<String, String> {
    // Skip the | or > indicator and any modifiers on the same line
    let _ = crate::utils::collect_until(source, |c| c == '\n');
    if source.current() == Some('\n') {
        source.next();
    }

    let mut raw_lines: Vec<String> = Vec::new();
    let mut first_indent: Option<usize> = None;

    loop {
        if source.current().is_none() {
            break;
        }

        let st_line = source.save_state();
        let mut cur_indent = 0usize;
        while let Some(CHAR_SPACE) = source.current() {
            cur_indent += 1;
            source.next();
        }

        let cur_is_newline = source.current() == Some(CHAR_NEWLINE);
        source.restore_state(st_line);

        if first_indent.is_none() {
            if cur_is_newline {
                let _ = crate::utils::collect_until(source, |c| c == CHAR_NEWLINE);
                if source.current() == Some(CHAR_NEWLINE) {
                    source.next();
                }
                raw_lines.push(String::new());
                continue;
            } else {
                first_indent = Some(cur_indent);
            }
        } else if !cur_is_newline && cur_indent < first_indent.unwrap() {
            break;
        }

        let raw_line = crate::utils::collect_until(source, |c| c == CHAR_NEWLINE);
        if source.current() == Some(CHAR_NEWLINE) {
            source.next();
        }
        raw_lines.push(raw_line);
    }

    // Remove trailing empty lines
    while matches!(raw_lines.last(), Some(s) if s.is_empty()) {
        raw_lines.pop();
    }

    let fi = first_indent.unwrap_or(0);
    let mut norm_lines: Vec<String> = Vec::with_capacity(raw_lines.len());

    if is_folded {
        // Folded scalar: normalize indentation
        for l in raw_lines.iter() {
            if l.is_empty() {
                norm_lines.push(String::new());
            } else {
                let lead = l.chars().take_while(|&ch| ch == CHAR_SPACE).count();
                let strip = fi.min(lead);
                let stripped: String = l.chars().skip(strip).collect();
                norm_lines.push(stripped);
            }
        }
    } else {
        // Literal scalar: keep as-is
        norm_lines = raw_lines.clone();
    }

    // Join lines according to scalar type
    let mut escaped_parts: Vec<String> = Vec::with_capacity(norm_lines.len());
    for l in norm_lines.iter() {
        if l.is_empty() {
            escaped_parts.push(String::new());
        } else {
            let lead = l.chars().take_while(|&ch| ch == CHAR_SPACE).count();
            let strip = fi.min(lead);
            let stripped: String = l.chars().skip(strip).collect();
            escaped_parts.push(stripped);
        }
    }

    Ok(escaped_parts.join("\\n"))
}

/// Creates a block scalar node from parsed content
#[allow(dead_code)]
pub(crate) fn make_block_scalar_node(content: String, is_folded: bool) -> Node {
    let style = if is_folded {
        BlockStyle::Folded
    } else {
        BlockStyle::Literal
    };
    Node::Str(content, QuoteType::Unquoted, style)
}
