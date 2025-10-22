// Shared character constants used by parser and utils
pub const CHAR_DASH: char = '-';
pub const CHAR_DOT: char = '.';
pub const CHAR_HASH: char = '#';
pub const CHAR_LBRACE: char = '{';
pub const CHAR_RBRACE: char = '}';
pub const CHAR_LBRACKET: char = '[';
pub const CHAR_RBRACKET: char = ']';
pub const CHAR_QUESTION: char = '?';
pub const CHAR_COLON: char = ':';
pub const CHAR_NEWLINE: char = '\n';
pub const CHAR_NUL: char = '\0';
pub const CHAR_COMMA: char = ',';
pub const CHAR_DOUBLE_QUOTE: char = '"';
pub const CHAR_SINGLE_QUOTE: char = '\'';
pub const CHAR_LESS: char = '<';
pub const CHAR_GREATER: char = '>';
// Newly added frequently used characters
pub const CHAR_PIPE: char = '|';
pub const CHAR_ASTERISK: char = '*';
pub const CHAR_AMPERSAND: char = '&';
pub const CHAR_SPACE: char = ' ';
pub const CHAR_BACKSLASH: char = '\\';
pub const CHAR_CARRIAGE_RETURN: char = '\r';
pub const CHAR_TAB: char = '\t';

// Token/string constants
pub const STR_EOF: &str = "<EOF>";
pub const STR_LITERAL_BLOCK: &str = "|";
pub const STR_FOLDED_BLOCK: &str = ">";
