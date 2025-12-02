/// Module: parser/document/mapping.rs
use crate::io::traits::ISource;
use crate::nodes::node::{BlockStyle, Node, QuoteType};
use crate::parser::document::mapping_tokens::parse_mapping_with_tokens;
use crate::parser::document::parse_value;
use crate::parser::document::sequence_tokens::parse_sequence_with_tokens;
use crate::parser::document::tokens::value::parse_value_with_tokens;

/// Parses a YAML mapping (dictionary) with the specified indentation level.
///
/// Processes key-value pairs, handling complex keys, nested mappings,
/// comments, and proper indentation. Determines appropriate quoting
/// for keys and values based on content safety rules.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The expected indentation level for mapping entries
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing a Mapping Node or an error string
pub(crate) fn parse_mapping(
    source: &mut dyn ISource,
    _indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    /// Checks if a string value can be safely represented as plain (unquoted) YAML.
    ///
    /// Returns false if the string contains characters that require quoting
    /// or has leading/trailing spaces that would be lost in plain format.
    fn is_plain_safe_value(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        if s.starts_with(' ') || s.ends_with(' ') {
            return false;
        }
        if s.contains(['\n', '\r']) {
            return false;
        }
        let disallowed = [
            '#', '[', ']', '{', '}', '&', '*', '!', '|', '>', '"', '`', '%', '@', '\\',
        ];
        if s.chars().any(|ch| disallowed.contains(&ch)) {
            return false;
        }
        true
    }
    /// Checks if a string key can be safely represented as plain (unquoted) YAML.
    ///
    /// Similar to is_plain_safe_value but additionally excludes colons which
    /// have special meaning in YAML key-value syntax.
    fn is_plain_safe_key(s: &str) -> bool {
        is_plain_safe_value(s) && !s.contains(':')
    }

    use crate::parser::lexer::Token;
    use crate::parser::token_stream::TokenStream;

    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut decorators: Option<(Option<String>, Option<String>, Option<String>)> = None; // (Tag, Anchor, Alias)
    let mut stream = TokenStream::new(source, directives)?;

    loop {
        let token = match stream.current() {
            Some(t) => t,
            None => break,
        };

        match token {
            Token::Tag(tag) => {
                decorators.get_or_insert((Some(tag.clone()), None, None));
                stream.next()?;
            }
            Token::Anchor(anchor) => {
                decorators.get_or_insert((None, Some(anchor.clone()), None));
                stream.next()?;
            }
            Token::Alias(alias) => {
                decorators.get_or_insert((None, None, Some(alias.clone())));
                stream.next()?;
            }
            Token::Plain(_s) | Token::SingleQuoted(_s) | Token::DoubleQuoted(_s) => {
                let mut key_node = match token {
                    Token::Plain(s) => Node::Str(s.clone(), QuoteType::Unquoted, BlockStyle::None),
                    Token::SingleQuoted(s) => {
                        Node::Str(s.clone(), QuoteType::Single, BlockStyle::None)
                    }
                    Token::DoubleQuoted(s) => {
                        Node::Str(s.clone(), QuoteType::Double, BlockStyle::None)
                    }
                    _ => unreachable!(),
                };
                if let Some((tag, anchor, alias)) = decorators.take() {
                    if let Some(t) = tag {
                        key_node = Node::Tagged(Box::new(key_node), t);
                    }
                    if let Some(a) = anchor {
                        key_node = Node::Anchored(Box::new(key_node), a);
                    }
                    if let Some(al) = alias {
                        key_node = Node::Alias(al);
                    }
                }
                stream.next()?;
                let value_node = match stream.current() {
                    Some(Token::Colon) => {
                        stream.next()?;
                        parse_value_with_tokens(&mut stream, directives)?
                    }
                    Some(Token::Newline) => Node::None,
                    Some(Token::Comma) => {
                        stream.next()?;
                        parse_value_with_tokens(&mut stream, directives)?
                    }
                    Some(Token::Dash) => parse_value_with_tokens(&mut stream, directives)?,
                    Some(Token::FlowMappingStart) | Some(Token::FlowSequenceStart) => {
                        parse_value_with_tokens(&mut stream, directives)?
                    }
                    Some(Token::Comment(_)) => {
                        stream.next()?;
                        parse_value_with_tokens(&mut stream, directives)?
                    }
                    Some(Token::Eof) | None => Node::None,
                    _ => parse_value_with_tokens(&mut stream, directives)?,
                };
                pairs.push((key_node, value_node));
            }
            Token::FlowMappingStart => {
                stream.next()?;
            }
            Token::Indent(_) | Token::Newline => {
                stream.next()?;
            }
            Token::Comment(_) => {
                stream.next()?;
            }
            Token::DocumentStart | Token::DocumentEnd | Token::Directive(_) | Token::Eof => {
                break;
            }
            _ => {
                stream.next()?;
            }
        }
    }
    Ok(Node::Mapping(pairs))
}
