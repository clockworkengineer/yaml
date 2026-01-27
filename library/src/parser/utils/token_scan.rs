/// Shared generic function for scanning tokens with a leading character and a set of delimiters.
/// Used to reduce duplication in scan_tag, scan_anchor, scan_alias, etc.
pub fn scan_token_with_leading<F, G, T>(
    source: &mut dyn crate::io::traits::ISource,
    consume_leading: F,
    mut is_delimiter: G,
    token_constructor: fn(String) -> T,
    allow_trailing_colon: bool,
    empty_error: &'static str,
) -> Result<T, crate::error::YamlError>
where
    F: Fn(&mut dyn crate::io::traits::ISource),
    G: FnMut(&mut dyn crate::io::traits::ISource, char) -> bool,
{
    consume_leading(source);
    let mut name = String::new();
    while let Some(ch) = source.current() {
        if is_delimiter(source, ch) {
            break;
        }
        name.push(ch);
        source.next();
    }
    if allow_trailing_colon && name.ends_with(':') {
        name.pop();
    }
    if name.is_empty() {
        return Err(crate::parser::document::error_builder::syntax_error(
            source,
            empty_error,
        ));
    }
    Ok(token_constructor(name))
}
