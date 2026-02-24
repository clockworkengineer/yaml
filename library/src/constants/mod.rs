//! YAML Library - Constants Module
//!
//! This module defines character and string constants used throughout the YAML parser and emitter.
//!
//! Copyright (c) 2026 YAML Library Contributors
//! License: MIT OR Apache-2.0
//!
//! Source: library/src/constants/mod.rs
//!
//! # Overview
//! This module centralizes all character and string constants for YAML syntax, improving code clarity and maintainability.
//!
//! # Usage
//! Use these constants when parsing or emitting YAML to avoid magic values and ensure consistency.
//!
//! ---
//!
//! Module: constants/mod.rs

pub const CHAR_DASH: char = '-';
pub const CHAR_HASH: char = '#';
pub const CHAR_LBRACE: char = '{';
pub const CHAR_RBRACE: char = '}';
pub const CHAR_LBRACKET: char = '[';
pub const CHAR_RBRACKET: char = ']';
pub const CHAR_COLON: char = ':';
pub const CHAR_NEWLINE: char = '\n';
pub const CHAR_COMMA: char = ',';
pub const CHAR_DOUBLE_QUOTE: char = '"';
pub const CHAR_SINGLE_QUOTE: char = '\'';

pub const CHAR_ASTERISK: char = '*';
pub const CHAR_AMPERSAND: char = '&';
pub const CHAR_SPACE: char = ' ';
pub const CHAR_BACKSLASH: char = '\\';
pub const CHAR_CARRIAGE_RETURN: char = '\r';
pub const CHAR_TAB: char = '\t';

pub const CHAR_DOT: char = '.';
pub const CHAR_QUESTION_MARK: char = '?';
pub const CHAR_PERCENT: char = '%';
pub const CHAR_EXCLAMATION: char = '!';
pub const CHAR_VERTICAL_BAR: char = '|';
pub const CHAR_GREATER_THAN: char = '>';

pub const STR_LITERAL_BLOCK: &str = "|";
pub const STR_FOLDED_BLOCK: &str = ">";
pub const STR_DOC_START: &str = "---";
pub const STR_DOC_END: &str = "...";
// ...existing code...
