/// Centralized error helpers for the lexer
pub fn syntax_error<S: AsRef<str>>(source: &mut dyn crate::io::traits::ISource, msg: S) -> crate::error::YamlError {
    crate::parser::document::error_builder::syntax_error(source, msg.as_ref())
}

pub fn forbidden_error<S1: AsRef<str>, S2: AsRef<str>>(source: &mut dyn crate::io::traits::ISource, what: S1, context: S2) -> crate::error::YamlError {
    crate::parser::document::error_builder::forbidden_error(source, what.as_ref(), context.as_ref())
}
