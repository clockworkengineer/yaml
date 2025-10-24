// Centralized error message strings for the YAML library

// Parser error message constants
pub const ERR_EXPECT_COLON_INLINE_MAPPING: &str = "Expected ':' in inline mapping";
pub const ERR_EOF_INLINE_MAPPING: &str = "Unexpected end of input in inline mapping";
pub const ERR_UNEXPECTED_CHAR_INLINE_MAPPING_PREFIX: &str =
    "Unexpected character in inline mapping: ";
pub const ERR_EOF_INLINE_SEQUENCE: &str = "Unexpected end of input in inline sequence";
pub const ERR_UNEXPECTED_CHAR_INLINE_SEQUENCE_PREFIX: &str =
    "Unexpected character in inline sequence: ";
// pub const ERR_EXPECT_SEQUENCE_ITEM: &str =
//     "Expected sequence item starting with CHAR_DASH, got '{}'";
// pub const ERR_EOF_EXPLICIT_PAIR: &str = "Unexpected end of input while parsing explicit pair value";
pub const ERR_UNEXPECTED_CHAR_PREFIX: &str = "Unexpected character: ";
pub const ERR_UNSUPPORTED_NODE_TYPE: &str = "Unsupported node type";
// pub const ERR_EXPECT_DOCUMENTS_NODE: &str = "Expected Documents node";
// pub const ERR_NODE_NOT_DOCUMENTS_ARRAY: &str = "Node is not a Document or Array of Documents";
pub const ERR_EXPECT_QUOTE_FORMAT: &str = "Expected quote, found '{}'";
pub const ERR_UNEXPECTED_EOF_AFTER_ANCHOR: &str = "Unexpected end of input after anchor";
pub const ERR_UNEXPECTED_EOF_EXPECTING_QUOTE: &str = "Unexpected EOF while expecting a quote";
pub const ERR_UNTERMINATED_QUOTED_FLOW: &str = "Unterminated quoted flow scalar";
pub const ERR_EMPTY_ALIAS_NAME: &str = "Empty alias name";
pub const ERR_EMPTY_ANCHOR_NAME: &str = "Empty anchor name";
pub const ERR_DUPLICATE_ANCHOR_PREFIX: &str = "Duplicate anchor name: ";
pub const ERR_UNDEFINED_ANCHOR_PREFIX: &str = "Undefined anchor reference: ";
