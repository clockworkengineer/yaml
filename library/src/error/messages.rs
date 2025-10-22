// Centralized error message strings for the YAML library

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