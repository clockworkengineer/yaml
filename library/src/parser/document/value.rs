//! Module: parser/document/value.rs

use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::document::helpers::{parse_error, parse_quoted_scalar, skip_whitespace};
use crate::parser::document::inline::{parse_inline_mapping, parse_inline_sequence};
use crate::parser::document::scalar::parse_scalar;
use crate::utils::*;


fn try_coerce_tag(tag: &str, node: Node) -> Option<Node> {


    match tag {
        "!!str" | "!str" => {


            let s = match node {
                Node::Str(s, _, _) => s,
                Node::Number(Numeric::Integer(i)) => i.to_string(),
                Node::Number(Numeric::Float(f)) => f.to_string(),
                Node::Boolean(b) => b.to_string(),
                Node::None => String::new(),
                _ => return None,
            };


            return Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None));
        }
        "!!int" | "!int" => {
            match node {
                Node::Number(Numeric::Integer(i)) => {
                    return Some(Node::Number(Numeric::Integer(i)));
                }
                Node::Number(Numeric::Float(f)) => {

                    if (f.trunc() - f).abs() < std::f64::EPSILON {
                        return Some(Node::Number(Numeric::Integer(f as i64)));
                    }
                    return None;
                }
                Node::Str(s, _, _) => {
                    if let Ok(i) = s.parse::<i64>() {
                        return Some(Node::Number(Numeric::Integer(i)));
                    }
                    return None;
                }
                _ => return None,
            }
        }
        "!!float" | "!float" => match node {
            Node::Number(Numeric::Float(f)) => return Some(Node::Number(Numeric::Float(f))),
            Node::Number(Numeric::Integer(i)) => {
                return Some(Node::Number(Numeric::Float(i as f64)));
            }
            Node::Str(s, _, _) => {
                if let Ok(f) = s.parse::<f64>() {
                    return Some(Node::Number(Numeric::Float(f)));
                }
                return None;
            }
            _ => return None,
        },
        "!!bool" | "!bool" => match node {
            Node::Boolean(b) => return Some(Node::Boolean(b)),
            Node::Str(s, _, _) => {
                let sl = s.to_ascii_lowercase();
                if sl == "true" {
                    return Some(Node::Boolean(true));
                }
                if sl == "false" {
                    return Some(Node::Boolean(false));
                }
                return None;
            }
            _ => return None,
        },
        "!!null" | "!null" => {
            return Some(Node::None);
        }
        "!!timestamp" | "!timestamp" => {


            match node {
                Node::Str(s, _, _) => {
                    return Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None));
                }
                Node::Number(Numeric::Integer(i)) => {
                    return Some(Node::Str(
                        i.to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ));
                }
                Node::Number(Numeric::Float(f)) => {
                    return Some(Node::Str(
                        f.to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ));
                }
                _ => return None,
            }
        }
        _ => return None,
    }
}

pub(crate) fn parse_value(source: &mut dyn ISource) -> Result<Node, String> {

    if source.current() == Some('!') {

        source.next();


        let rest = collect_until(source, |c| c == CHAR_SPACE || c == CHAR_NEWLINE);
        let tag = format!("!{}", rest);
        if tag.trim().is_empty() {
            return Err(parse_error(source, "Empty tag"));
        }
        skip_whitespace(source);

        let inner = match source.current() {
            Some(CHAR_LBRACE) => parse_inline_mapping(source)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim())
            }
            Some('-') => {


                let st = source.save_state();
                source.next();
                let next_ch = source.current();
                source.restore_state(st);

                if let Some(ch) = next_ch {
                    if source.is_whitespace(ch) || ch == CHAR_NEWLINE {
                        let nested_indent = source.get_current_indent_level();
                        crate::parser::document::parse_sequence(source, nested_indent)?
                    } else {

                        parse_value(source)?
                    }
                } else {
                    return Err(parse_error(source, ERR_UNEXPECTED_EOF_AFTER_ANCHOR));
                }
            }
            Some(_) => parse_value(source)?,
            None => return Err(parse_error(source, ERR_UNEXPECTED_EOF_AFTER_ANCHOR)),
        };


        if let Some(coerced) = try_coerce_tag(&tag, inner.clone()) {
            return Ok(coerced);
        }

        return Ok(Node::Tagged(Box::new(inner), tag));
    }

    if source.current() == Some(CHAR_ASTERISK) {
        source.next();
        let name = collect_until(source, |c| {
            c == CHAR_NEWLINE || c == CHAR_HASH || c.is_whitespace()
        });
        if name.trim().is_empty() {
            return Err(parse_error(source, ERR_EMPTY_ALIAS_NAME));
        }
        return Ok(Node::Alias(name));
    }

    if source.current() == Some(CHAR_AMPERSAND) {
        source.next();
        let name = collect_until(source, |c| {
            c == CHAR_SPACE || c == CHAR_NEWLINE || c == CHAR_HASH
        });
        if name.trim().is_empty() {
            return Err(parse_error(source, ERR_EMPTY_ANCHOR_NAME));
        }
        skip_whitespace(source);
        if source.current() == Some(CHAR_NEWLINE) {
            source.next();
            skip_whitespace(source);
            let nested_indent = source.get_current_indent_level();
            let node = crate::parser::document::parse_document_contents(source, nested_indent)?;
            return Ok(Node::Anchored(Box::new(node), name));
        }
        let node = match source.current() {
            Some(CHAR_LBRACE) => parse_inline_mapping(source)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim())
            }
            Some(_) => parse_value(source)?,
            None => return Err(parse_error(source, ERR_UNEXPECTED_EOF_AFTER_ANCHOR)),
        };
        return Ok(Node::Anchored(Box::new(node), name));
    }

    match source.current() {
        Some(CHAR_LBRACE) => parse_inline_mapping(source),
        Some(CHAR_LBRACKET) => parse_inline_sequence(source),
        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
            let raw = parse_quoted_scalar(source)?;
            let trimmed = raw.trim();
            if trimmed.starts_with(CHAR_DOUBLE_QUOTE)
                && trimmed.ends_with(CHAR_DOUBLE_QUOTE)
                && trimmed.contains('\n')
            {
                let parsed = parse_scalar(trimmed);
                return if let Node::Str(content, QuoteType::Double, _style) = parsed {
                    let had_indent_after_newline = trimmed.contains("\n ");
                    let had_explicit_escaped_nl_at_end = trimmed.ends_with("\\\n\"");
                    if had_indent_after_newline || had_explicit_escaped_nl_at_end {
                        Ok(Node::Str(content, QuoteType::Unquoted, BlockStyle::Literal))
                    } else {
                        Ok(Node::Str(content, QuoteType::Double, BlockStyle::None))
                    }
                } else {
                    Ok(parsed)
                };
            }
            Ok(parse_scalar(trimmed))
        }
        Some(_) => {
            let value = collect_until(source, |c| c == CHAR_NEWLINE || c == CHAR_HASH);
            let trimmed = value.trim();


            if trimmed.starts_with(STR_LITERAL_BLOCK) || trimmed.starts_with(STR_FOLDED_BLOCK) {


                let first_ch = trimmed.chars().next().unwrap();
                let rest = trimmed[1..].trim();
                let valid_header_rest = rest
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '+' || c == '-');
                if !valid_header_rest {

                    if !trimmed.is_empty() {
                        return Ok(parse_scalar(trimmed));
                    } else {
                        return Ok(Node::None);
                    }
                }
                let is_folded = first_ch == STR_FOLDED_BLOCK.chars().next().unwrap();

                let keep_trailing = trimmed.contains('+');
                if source.current() == Some(CHAR_NEWLINE) {
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
                            let _ = collect_until(source, |c| c == CHAR_NEWLINE);
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

                    let raw_line = collect_until(source, |c| c == CHAR_NEWLINE);
                    if source.current() == Some(CHAR_NEWLINE) {
                        source.next();
                    }
                    raw_lines.push(raw_line);
                }


                if !keep_trailing {
                    while matches!(raw_lines.last(), Some(s) if s.is_empty()) {
                        raw_lines.pop();
                    }
                }

                let fi = first_indent.unwrap_or(0);
                let mut norm_lines: Vec<String> = Vec::with_capacity(raw_lines.len());
                if is_folded {
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
                    norm_lines = raw_lines.clone();
                }

                let out = if is_folded {
                    let mut out = String::new();
                    let mut i = 0usize;
                    while i < norm_lines.len() {
                        let line = &norm_lines[i];
                        if line.is_empty() {
                            out.push(CHAR_NEWLINE);
                            i += 1;
                            continue;
                        }
                        let is_indented = line.starts_with(' ');
                        if !is_indented {
                            let mut j = i;
                            let mut first = true;
                            while j < norm_lines.len() {
                                let l = &norm_lines[j];
                                if l.is_empty() || l.starts_with(' ') {
                                    break;
                                }
                                if !first {
                                    out.push(CHAR_SPACE);
                                }
                                out.push_str(l);
                                first = false;
                                j += 1;
                            }
                            out.push(CHAR_NEWLINE);
                            i = j;
                        } else {
                            out.push_str(line);
                            out.push(CHAR_NEWLINE);
                            i += 1;
                        }
                    }
                    if out.ends_with('\n') {
                        out.pop();
                    }
                    out
                } else {
                    let mut out = String::new();
                    for (idx, l) in norm_lines.iter().enumerate() {
                        out.push_str(l);
                        if idx + 1 < norm_lines.len() {
                            out.push(CHAR_NEWLINE);
                        }
                    }
                    out
                };

                return Ok(Node::Str(
                    out,
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::Literal,
                ));
            }
            if !trimmed.is_empty() {
                Ok(parse_scalar(trimmed))
            } else {
                Ok(Node::None)
            }
        }
        None => Ok(Node::None),
    }
}
