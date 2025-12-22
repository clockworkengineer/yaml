/// Module: parser/document/sequence.rs
// ...existing code...
use crate::parser::token_stream::TokenStream;

/// Parses a YAML sequence (array) with the specified indentation level.
///
/// Processes sequence items marked with '-' at the beginning of lines,
/// handling nested sequences, comments, and document boundaries.
/// Maintains proper indentation tracking for nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The expected indentation level for sequence items
/// Module: parser/document/sequence.rs
use crate::io::traits::ISource;

/// Parses a YAML sequence (array) with the specified indentation level.
pub(crate) fn parse_sequence(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<crate::nodes::node::Node, String> {
    let mut stream = TokenStream::new(source, directives, false)?;
    // NOTE: parse_sequence is only used in legacy paths, so we pass a default context
    let ctx = crate::parser::document::context::ParsingContext::default();
    crate::parser::document::tokens::sequence::parse_sequence_with_tokens(
        &mut stream,
        indent_level,
        directives,
        &ctx,
        0,
    )
}

// Helper for nested sequence parsing to avoid double mutable borrow
#[allow(dead_code)]
fn parse_sequence_inner(
    stream: &mut TokenStream,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<crate::nodes::node::Node, String> {
    let mut items = Vec::new();
    while let Some(token) = stream.current() {
        match token {
            crate::parser::lexer::Token::Newline | crate::parser::lexer::Token::Comment(_) => {
                stream.next()?;
                continue;
            }
            crate::parser::lexer::Token::Indent(level) => {
                if *level < indent_level {
                    break;
                }
                stream.next()?;
                continue;
            }
            crate::parser::lexer::Token::Dash => {
                stream.next()?;
                stream.skip_whitespace()?;
                match stream.current() {
                    Some(crate::parser::lexer::Token::Dash) => {
                        let nested = parse_sequence_inner(stream, indent_level + 1, directives)?;
                        items.push(nested);
                        continue;
                    }
                    Some(crate::parser::lexer::Token::FlowSequenceStart)
                    | Some(crate::parser::lexer::Token::FlowMappingStart) => {
                        use crate::parser::document::tokens::value::parse_value_with_tokens;
                        let value = parse_value_with_tokens(stream, directives, 0)?;
                        items.push(value);
                        continue;
                    }
                    _ => {
                        use crate::parser::document::tokens::value::parse_value_with_tokens;
                        let value = parse_value_with_tokens(stream, directives, 0)?;
                        items.push(value);
                        continue;
                    }
                }
            }
            crate::parser::lexer::Token::Eof
            | crate::parser::lexer::Token::DocumentEnd
            | crate::parser::lexer::Token::DocumentStart => {
                break;
            }
            _ => {
                stream.next()?;
            }
        }
    }
    Ok(crate::nodes::node::Node::Array(items))
}
