//! Buffer Source for Decoded Input
//!
//! Provides a memory buffer implementation for reading YAML or JSON data from bytes.
//! Implements the `ISource` trait for traversing and reading byte content from memory.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::ISource;

/// A memory buffer implementation for reading JSON data from bytes.
/// Provides functionality to traverse and read byte content from memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    buffer: Vec<u8>,
    /// Current reading position in the buffer
    position: usize,
    /// Current column position in the buffer
    column: usize,
    /// Current line position in the buffer
    line: usize,
}

impl Buffer {
    /// Creates a new Buffer instance with the specified byte slice.
    ///
    /// # Arguments
    /// * `to_add` - The byte slice to initialize the buffer with
    ///
    /// # Returns
    /// A new Buffer containing the provided bytes
    pub fn new(to_add: &[u8]) -> Self {
        Self {
            buffer: to_add.to_vec(),
            position: 0,
            column: 0,
            line: 0,
        }
    }
    /// Converts the buffer content to a String.
    ///
    /// # Returns
    /// A String containing UTF-8 interpretation of the buffer bytes.
    pub fn to_string(&self) -> String {
        // Use shared helper for conversion
        String::from_utf8_lossy(&self.buffer).into_owned()
    }
}

impl ISource for Buffer {
    /// Moves to the next character in the buffer
    fn next(&mut self) {
        if !self.more() {
            return;
        }

        let current_byte = self.buffer[self.position];
        self.position += 1;

        // Handle line breaks: both \n (LF) and \r\n (CRLF) and \r (CR)
        match current_byte {
            b'\n' => {
                self.line += 1;
                self.column = 0;
            }
            b'\r' => {
                // Check if this is part of CRLF sequence
                if self.more() && self.buffer[self.position] == b'\n' {
                    // CRLF - don't increment line yet, let the \n handle it
                    // But don't increment column either
                } else {
                    // Standalone CR - treat as line break
                    self.line += 1;
                    self.column = 0;
                }
            }
            _ => {
                self.column += 1;
            }
        }
    }
    /// Returns the current character at the buffer position
    fn current(&mut self) -> Option<char> {
        if self.more() {
            Some(self.buffer[self.position] as char)
        } else {
            None
        }
    }
    /// Checks if there are more characters to read
    fn more(&mut self) -> bool {
        self.position < self.buffer.len()
    }
    /// Resets the buffer position to the start
    fn reset(&mut self) {
        self.position = 0;
    }

    fn get_current_indent_level(&self) -> usize {
        self.column
    }

    fn save_state(&mut self) -> crate::io::traits::SaveState {
        crate::io::traits::SaveState {
            pos: self.position as u64,
            current_byte: if self.more() {
                Some(self.buffer[self.position])
            } else {
                None
            },
            column: self.column,
            line: self.line,
        }
    }

    fn restore_state(&mut self, state: crate::io::traits::SaveState) {
        self.position = state.pos as usize;
        self.column = state.column;
        self.line = state.line;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_source_buffer_works() {
        let source = Buffer::new(String::from("i32e").as_bytes());
        assert_eq!(source.to_string(), "i32e");
    }
    #[test]
    fn read_character_from_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        match source.current() {
            Some('i') => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn move_to_next_character_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        source.next();
        match source.current() {
            Some('3') => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn move_to_last_character_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        while source.more() {
            source.next()
        }
        match source.current() {
            None => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn reset_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        while source.more() {
            source.next()
        }
        source.reset();
        match source.current() {
            Some('i') => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn create_empty_buffer_works() {
        let source = Buffer::new(&[]);
        assert_eq!(source.to_string(), "");
    }
    #[test]
    fn handle_non_utf8_content() {
        let source = Buffer::new(&[0xFF]);
        assert_eq!(source.to_string(), String::from_utf8_lossy(&[0xFF]));
    }
    #[test]
    fn more_returns_correct_at_boundaries() {
        let mut source = Buffer::new(String::from("a").as_bytes());
        assert!(source.more());
        source.next();
        assert!(!source.more());
    }
    #[test]
    fn multiple_next_calls_work() {
        let mut source = Buffer::new(String::from("abc").as_bytes());
        source.next();
        source.next();
        match source.current() {
            Some('c') => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn save_restore_works() {
        let mut source = Buffer::new(String::from("abc").as_bytes());

        let s0 = source.save_state();
        source.next();
        source.restore_state(s0);
        match source.current() {
            Some('a') => assert!(true),
            _ => assert!(false),
        }

        source.next();
        let s = source.save_state();
        source.next();
        assert_eq!(source.current(), Some('c'));
        source.restore_state(s);
        assert_eq!(source.current(), Some('b'));
    }
    #[test]
    fn get_current_indent_level_works() {
        let mut source = Buffer::new(String::from("  abc").as_bytes());
        source.next();
        source.next();
        assert_eq!(source.get_current_indent_level(), 2);
    }

    #[test]
    fn buffer_eof_after_consumption() {
        let mut source = Buffer::new(String::from("xy").as_bytes());

        source.next();
        source.next();
        assert_eq!(source.current(), None);
        assert!(!source.more());
    }

    #[test]
    fn buffer_next_safe_at_eof() {
        let mut source = Buffer::new(String::from("a").as_bytes());
        assert_eq!(source.current(), Some('a'));
        source.next();
        assert_eq!(source.current(), None);
        source.next();
        assert_eq!(source.current(), None);
        assert!(!source.more());
    }

    #[test]
    fn buffer_reset_restores_after_eof() {
        let mut source = Buffer::new(String::from("z").as_bytes());
        assert_eq!(source.current(), Some('z'));
        source.next();
        assert_eq!(source.current(), None);
        source.reset();
        assert_eq!(source.current(), Some('z'));
        assert!(source.more());
    }
    #[test]
    fn buffer_handles_only_newlines() {
        let mut source = Buffer::new(b"\n\n\n");
        assert_eq!(source.line, 0);
        assert_eq!(source.column, 0);
        for i in 0..3 {
            assert_eq!(source.current(), Some('\n'));
            source.next();
            assert_eq!(source.line, i + 1);
            assert_eq!(source.column, 0);
        }
        assert_eq!(source.current(), None);
    }

    #[test]
    fn buffer_handles_mixed_line_endings() {
        let mut source = Buffer::new(b"a\r\nb\rc\nd");
        assert_eq!(source.current(), Some('a'));
        source.next(); // '\r'
        assert_eq!(source.current(), Some('\r'));
        source.next(); // '\n' (part of CRLF)
        assert_eq!(source.current(), Some('\n'));
        source.next(); // 'b'
        assert_eq!(source.current(), Some('b'));
        source.next(); // '\r'
        assert_eq!(source.current(), Some('\r'));
        source.next(); // 'c'
        assert_eq!(source.current(), Some('c'));
        source.next(); // '\n'
        assert_eq!(source.current(), Some('\n'));
        source.next(); // 'd'
        assert_eq!(source.current(), Some('d'));
        source.next();
        assert_eq!(source.current(), None);
    }

    #[test]
    fn buffer_save_restore_multiple_points() {
        let mut source = Buffer::new(b"abcde");
        let s0 = source.save_state();
        source.next();
        let s1 = source.save_state();
        source.next();
        let s2 = source.save_state();
        source.next();
        assert_eq!(source.current(), Some('d'));
        source.restore_state(s1);
        assert_eq!(source.current(), Some('b'));
        source.restore_state(s2);
        assert_eq!(source.current(), Some('c'));
        source.restore_state(s0);
        assert_eq!(source.current(), Some('a'));
    }

    #[test]
    fn buffer_reset_after_partial_read() {
        let mut source = Buffer::new(b"xyz");
        source.next();
        assert_eq!(source.current(), Some('y'));
        source.reset();
        assert_eq!(source.current(), Some('x'));
    }

    #[test]
    fn buffer_handles_large_input() {
        let data = vec![b'a'; 10_000];
        let mut source = Buffer::new(&data);
        let mut count = 0;
        while source.more() {
            assert_eq!(source.current(), Some('a'));
            source.next();
            count += 1;
        }
        assert_eq!(count, 10_000);
    }
}
