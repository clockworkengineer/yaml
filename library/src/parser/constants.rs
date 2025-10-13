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

// Parser error message constants
pub const ERR_EXPECT_COLON_INLINE_MAPPING: &str = "Expected ':' in inline mapping";
pub const ERR_EOF_INLINE_MAPPING: &str = "Unexpected end of input in inline mapping";
pub const ERR_UNEXPECTED_CHAR_INLINE_MAPPING_PREFIX: &str =
    "Unexpected character in inline mapping: ";
pub const ERR_EOF_INLINE_SEQUENCE: &str = "Unexpected end of input in inline sequence";
pub const ERR_UNEXPECTED_CHAR_INLINE_SEQUENCE_PREFIX: &str =
    "Unexpected character in inline sequence: ";
pub const ERR_EXPECT_SEQUENCE_ITEM: &str =
    "Expected sequence item starting with CHAR_DASH, got '{}'";
pub const ERR_EOF_EXPLICIT_PAIR: &str = "Unexpected end of input while parsing explicit pair value";
pub const ERR_UNEXPECTED_CHAR_PREFIX: &str = "Unexpected character: ";
