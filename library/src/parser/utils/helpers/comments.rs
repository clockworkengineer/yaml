//! Comment parsing helpers for YAML parsing.
//!
//! Provides token-based comment consumption from a `TokenStream`.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Parses a comment line from the source.
///
/// DRY ENTRY POINT: All comment parsing must use this function.
/// Usage: Call this when you need to parse a comment token from the stream.
///
/// Consumes a Comment token from the TokenStream and returns its content.
/// Returns an empty string if the current token is not a Comment.
#[allow(dead_code)]
pub(crate) fn parse_comment_token(stream: &mut crate::parser::token_stream::TokenStream) -> String {
    use crate::parser::lexer::Token;
    match stream.current() {
        Some(Token::Comment(s)) => {
            let comment = s.clone();
            let _ = stream.next();
            comment.trim().to_string()
        }
        _ => String::new(),
    }
}
