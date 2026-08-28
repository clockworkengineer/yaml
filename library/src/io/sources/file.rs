//! File Source for Decoded Input
//!
//! Provides a file-based source for reading YAML or JSON data from disk.
//! Implements the `ISource` trait and file operations for traversing and reading byte content from files.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::{ICharStream, IIndentationAware, IStatefulStream};

impl ICharStream for File {
    fn next(&mut self) {
        // Read next byte
        let mut byte1 = [0u8; 1];

        // Update position for current character before moving
        if self.current_byte.is_some() {
            self.column += 1;
            if self.current_byte.unwrap() == b'\n' {
                self.line += 1;
                self.column = 0;
            }
        }

        if self.file.read(&mut byte1).unwrap_or(0) == 1 {
            if byte1[0] == b'\r' {
                // Check if followed by \n (CRLF sequence)
                let mut byte2 = [0u8; 1];
                match self.file.read(&mut byte2) {
                    Ok(1) if byte2[0] == b'\n' => {
                        // CRLF - treat as single \n
                        self.current_byte = Some(b'\n');
                        self.column = 0; // Newlines have column 0
                    }
                    Ok(1) => {
                        // Standalone CR
                        self.current_byte = Some(byte1[0]);
                        self.file.seek(SeekFrom::Current(-1)).unwrap();
                    }
                    _ => {
                        self.current_byte = Some(byte1[0]);
                    }
                }
            } else {
                self.current_byte = Some(byte1[0]);
            }
        } else {
            self.current_byte = None;
        }
    }
    fn current(&mut self) -> Option<char> {
        self.current_byte.map(|b| b as char)
    }

    fn more(&mut self) -> bool {
        self.current_byte.is_some()
    }

    fn reset(&mut self) {
        if self.file.seek(SeekFrom::Start(0)).is_ok() {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                Some(byte[0])
            } else {
                None
            };
            self.column = 0;
            self.line = 0;
        }
    }
}

impl IIndentationAware for File {
    fn get_current_indent_level(&self) -> usize {
        self.column
    }
}

impl IStatefulStream for File {
    fn save_state(&mut self) -> crate::io::traits::SaveState {
        let pos = self.file.stream_position().unwrap_or(0);
        crate::io::traits::SaveState {
            pos,
            current_byte: self.current_byte,
            column: self.column,
            line: self.line,
        }
    }

    fn restore_state(&mut self, state: crate::io::traits::SaveState) {
        let _ = self.file.seek(SeekFrom::Start(state.pos));
        self.current_byte = state.current_byte;
        self.column = state.column;
        self.line = state.line;
    }
}
use crate::io::util::read_all;
use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom};

/// File

pub struct File {
    file: StdFile,
    current_byte: Option<u8>,
    column: usize,
    line: usize,
}
impl File {
    /// Reads the entire file into a Vec<u8> using shared helper
    pub fn read_all_bytes(&mut self) -> std::io::Result<Vec<u8>> {
        read_all(&mut self.file)
    }
}

impl File {
    /// new
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut file = StdFile::open(path)?;

        // Read first byte and handle CRLF
        let mut first = [0u8; 1];
        let current_byte = if file.read(&mut first)? == 1 {
            if first[0] == b'\r' {
                // Check if this is CRLF
                let mut next = [0u8; 1];
                if file.read(&mut next)? == 1 && next[0] == b'\n' {
                    // CRLF - treat as \n
                    Some(b'\n')
                } else {
                    // Standalone CR or CR followed by non-LF
                    if next[0] != 0 {
                        file.seek(SeekFrom::Current(-1))?;
                    }
                    Some(first[0])
                }
            } else {
                Some(first[0])
            }
        } else {
            None
        };

        Ok(Self {
            file,
            current_byte,
            column: 0,
            line: 0,
        })
    }
    pub fn next(&mut self) {
        ICharStream::next(self)
    }
    pub fn current(&mut self) -> Option<char> {
        ICharStream::current(self)
    }
    pub fn more(&mut self) -> bool {
        ICharStream::more(self)
    }
    pub fn reset(&mut self) {
        ICharStream::reset(self)
    }
    pub fn get_current_indent_level(&self) -> usize {
        IIndentationAware::get_current_indent_level(self)
    }
    pub fn save_state(&mut self) -> crate::io::traits::SaveState {
        IStatefulStream::save_state(self)
    }
    pub fn restore_state(&mut self, state: crate::io::traits::SaveState) {
        IStatefulStream::restore_state(self, state)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
    use crate::parse;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestFile {
        path: String,
    }

    impl TestFile {
        fn new(content: &[u8]) -> Self {
            let id = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = format!("test_temp_file_{}.yaml", id);
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .unwrap();
            file.write_all(content).unwrap();
            Self { path }
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    #[test]
    fn test_file_new_and_current() {
        let test_file = TestFile::new(b"abc");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.current(), Some('a'));
    }

    #[test]
    fn test_file_next_and_more() {
        let test_file = TestFile::new(b"ab");
        let mut file = File::new(&test_file.path).unwrap();
        assert!(file.more());
        assert_eq!(file.current(), Some('a'));
        file.next();
        assert_eq!(file.current(), Some('b'));
        assert!(file.more());
        file.next();
        assert_eq!(file.current(), None);
        assert!(!file.more());
    }

    #[test]
    fn test_file_reset() {
        let test_file = TestFile::new(b"xy");
        let mut file = File::new(&test_file.path).unwrap();
        file.next();
        assert_eq!(file.current(), Some('y'));
        file.reset();
        assert_eq!(file.current(), Some('x'));
    }

    #[test]
    fn test_file_save_restore() {
        let test_file = TestFile::new(b"123\r\n456");
        let mut file = File::new(&test_file.path).unwrap();

        assert_eq!(file.current(), Some('1'));
        let s1 = file.save_state();
        file.next();
        let s2 = file.save_state();
        file.next();
        let s3 = file.save_state();
        file.next();
        let s_newline = file.save_state();
        file.next();
        assert_eq!(file.current(), Some('4'));

        file.restore_state(s_newline);
        assert_eq!(file.current(), Some('\n'));
        file.restore_state(s3);
        assert_eq!(file.current(), Some('3'));
        file.restore_state(s2);
        assert_eq!(file.current(), Some('2'));
        file.restore_state(s1);
        assert_eq!(file.current(), Some('1'));
        assert_eq!(file.get_current_indent_level(), 0);
    }

    #[test]
    fn test_file_get_current_indent_level() {
        let test_file = TestFile::new(b"abc\ndef");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.get_current_indent_level(), 0);
        file.next();
        assert_eq!(file.get_current_indent_level(), 1);
        file.next();
        assert_eq!(file.get_current_indent_level(), 2);
        file.next();
        assert_eq!(file.get_current_indent_level(), 3);
        file.next();
        assert_eq!(file.get_current_indent_level(), 0);
    }

    #[test]
    fn test_file_new_empty_file() {
        let test_file = TestFile::new(b"");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.current(), None);
        assert!(!file.more());
    }

    #[test]
    fn test_file_handles_crlf_newlines() {
        let test_file = TestFile::new(b"ab\r\ncd\r\nef");
        let mut file = File::new(&test_file.path).unwrap();

        assert_eq!(file.current(), Some('a'));
        assert_eq!(file.get_current_indent_level(), 0);

        file.next();
        assert_eq!(file.current(), Some('b'));
        assert_eq!(file.get_current_indent_level(), 1);

        file.next();
        assert_eq!(file.current(), Some('\n'));
        assert_eq!(file.get_current_indent_level(), 0);

        file.next();
        assert_eq!(file.current(), Some('c'));
        assert_eq!(file.get_current_indent_level(), 0);

        file.next();
        assert_eq!(file.current(), Some('d'));
        assert_eq!(file.get_current_indent_level(), 1);

        file.next();
        assert_eq!(file.current(), Some('\n'));
        assert_eq!(file.get_current_indent_level(), 0);

        file.next();
        assert_eq!(file.current(), Some('e'));
        assert_eq!(file.get_current_indent_level(), 0);

        file.next();
        assert_eq!(file.current(), Some('f'));
        assert_eq!(file.get_current_indent_level(), 1);

        file.next();
        assert_eq!(file.current(), None);
    }

    #[test]
    fn test_file_eof_after_consumption() {
        let test_file = TestFile::new(b"xy");
        let mut file = File::new(&test_file.path).unwrap();

        file.next();
        file.next();
        assert_eq!(file.current(), None);
        assert!(!file.more());
    }

    #[test]
    fn test_file_next_safe_at_eof() {
        let test_file = TestFile::new(b"a");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.current(), Some('a'));
        file.next();
        assert_eq!(file.current(), None);

        file.next();
        assert_eq!(file.current(), None);
        assert!(!file.more());
    }

    #[test]
    fn test_file_reset_restores_after_eof() {
        let test_file = TestFile::new(b"- [Sammy Sosa, 63, 0.288]");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.current(), Some('-'));
        file.next();
        file.next();
        assert_eq!(file.current(), Some('['));
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        file.next();
        assert_eq!(file.current(), None);
        file.reset();
        assert_eq!(file.current(), Some('-'));
        assert!(file.more());
    }
    #[test]
    fn test_file_parse_nested_sequences() {
        let test_file = TestFile::new(b"- [Sammy Sosa, 63, 0.288]");
        let mut file = File::new(&test_file.path).unwrap();
        let node = parse(&mut file).unwrap();
        assert_eq!(
            node,
            Node::Documents(vec![Node::Document(vec![Node::Array(vec![Node::Array(
                vec![
                    Node::Str(
                        "Sammy Sosa".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None
                    ),
                    Node::Number(Numeric::Integer(63)),
                    Node::Number(Numeric::Float(0.288))
                ]
            )])])])
        );
    }

    fn create_temp_file(data: &[u8], name: &str) -> String {
        let path = format!("test_temp_{}.txt", name);
        let mut f = StdFile::create(&path).unwrap();
        f.write_all(data).unwrap();
        path
    }

    #[test]
    fn file_handles_only_newlines() {
        let path = create_temp_file(b"\n\n\n", "only_newlines");
        let mut file = File::new(&path).unwrap();
        for _ in 0..3 {
            assert_eq!(file.current(), Some('\n'));
            file.next();
        }
        assert_eq!(file.current(), None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn file_handles_mixed_line_endings() {
        let path = create_temp_file(b"a\r\nb\rc\nd", "mixed_lines");
        let mut file = File::new(&path).unwrap();
        assert_eq!(file.current(), Some('a'));
        file.next(); // '\r'
        assert_eq!(file.current(), Some('\n'));
        file.next(); // 'b'
        assert_eq!(file.current(), Some('b'));
        file.next(); // '\r'
        assert_eq!(file.current(), Some('\r'));
        file.next(); // 'c'
        assert_eq!(file.current(), Some('c'));
        file.next(); // '\n'
        assert_eq!(file.current(), Some('\n'));
        file.next(); // 'd'
        assert_eq!(file.current(), Some('d'));
        file.next();
        assert_eq!(file.current(), None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn file_reset_after_partial_read() {
        let path = create_temp_file(b"xyz", "reset_partial");
        let mut file = File::new(&path).unwrap();
        file.next();
        assert_eq!(file.current(), Some('y'));
        file.reset();
        assert_eq!(file.current(), Some('x'));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn file_handles_large_input() {
        let data = vec![b'a'; 10_000];
        let path = create_temp_file(&data, "large_input");
        let mut file = File::new(&path).unwrap();
        let mut count = 0;
        while file.more() {
            assert_eq!(file.current(), Some('a'));
            file.next();
            count += 1;
        }
        assert_eq!(count, 10_000);
        fs::remove_file(path).unwrap();
    }
}
