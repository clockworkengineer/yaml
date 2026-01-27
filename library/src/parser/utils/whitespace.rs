// Utility functions for whitespace and indentation handling in the lexer
// Centralizes logic to support DRY refactoring

#[allow(dead_code)]
pub fn consume_horizontal_whitespace<F>(mut next_char: F) -> usize
where
    F: FnMut() -> Option<char>,
{
    let mut count = 0;
    while let Some(ch) = next_char() {
        if ch == ' ' || ch == '\t' {
            count += 1;
        } else {
            break;
        }
    }
    count
}

#[allow(dead_code)]
pub fn skip_horizontal_whitespace<F>(mut next_char: F)
where
    F: FnMut() -> Option<char>,
{
    while let Some(ch) = next_char() {
        if ch != ' ' && ch != '\t' {
            break;
        }
    }
}

#[allow(dead_code)]
pub fn peek_next_non_whitespace<F, R>(
    mut save_state: F,
    mut restore_state: F,
    mut next_char: R,
) -> Option<char>
where
    F: FnMut(),
    R: FnMut() -> Option<char>,
{
    save_state();
    while let Some(ch) = next_char() {
        if ch != ' ' && ch != '\t' {
            break;
        }
    }
    let result = next_char();
    restore_state();
    result
}
