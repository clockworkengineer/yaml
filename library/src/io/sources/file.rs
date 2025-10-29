use crate::io::traits::ISource;
use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom};

pub struct File {
    file: StdFile,
    current_byte: Option<u8>,
    column: usize,
    line: usize,
}

impl File {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut file = StdFile::open(path)?;
        // Read first byte and normalize CRLF to a single '\n' like `next()` does.
        let mut first = [0u8; 1];
        let has_first = file.read(&mut first)? == 1;
        let current_byte = if has_first {
            if first[0] == b'\r' {
                // Peek the next byte
                let mut next = [0u8; 1];
                if file.read(&mut next)? == 1 {
                    if next[0] == b'\n' {
                        // CRLF -> treat as single '\n'
                        Some(b'\n')
                    } else {
                        // Not CRLF: seek back one and keep '\r'
                        file.seek(SeekFrom::Current(-1))?;
                        Some(first[0])
                    }
                } else {
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
}

impl ISource for File {
    fn next(&mut self) {
        let mut byte1 = [0u8; 1];
        let mut byte2 = [0u8; 1];
        if self.current_byte.is_some() {
            self.column += 1;
            if self.current_byte.unwrap() == b'\n' {
                self.line += 1;
                self.column = 0;
            }
        }
        if self.file.read(&mut byte1).unwrap_or(0) == 1 {
            if byte1[0] == b'\r' {
                // Attempt to read the following byte; handle partial/EOF reads explicitly
                match self.file.read(&mut byte2) {
                    Ok(1) => {
                        // Treat \r\n as a single \n
                        if byte2[0] == b'\n' {
                            self.line += 1;
                            self.column = 0;
                            self.current_byte = Some(b'\n');
                        } else {
                            self.current_byte = Some(byte1[0]);
                            self.file.seek(SeekFrom::Current(-1)).unwrap();
                        }
                    }
                    Ok(_) | Err(_) => {
                        // No next byte available; keep the '\r' as current
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

    // Use save_state/restore_state for restoring positions
    fn get_current_indent_level(&self) -> usize {
        self.column
    }

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
        // Restore underlying file cursor to the saved next-read position
        let _ = self.file.seek(SeekFrom::Start(state.pos));
        self.current_byte = state.current_byte;
        self.column = state.column;
        self.line = state.line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
    use crate::parse;
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
        // Validate save/restore works for multiple positions
        assert_eq!(file.current(), Some('1'));
        let s1 = file.save_state(); // at '1'
        file.next(); // move to '2'
        let s2 = file.save_state(); // at '2'
        file.next(); // move to '3'
        let s3 = file.save_state(); // at '3'
        file.next(); // move to '\n' (CRLF treated as single '\n')
        let s_newline = file.save_state(); // at '\n'
        file.next(); // move to '4'
        assert_eq!(file.current(), Some('4'));

        // restore to various saved positions
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
        file.next(); // now at '\n'
        assert_eq!(file.get_current_indent_level(), 3);
        file.next(); // should reset column to 0
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

        // 'a'
        assert_eq!(file.current(), Some('a'));
        assert_eq!(file.get_current_indent_level(), 0);

        // 'b'
        file.next();
        assert_eq!(file.current(), Some('b'));
        assert_eq!(file.get_current_indent_level(), 1);

        // '\n' (should reset column)
        file.next();
        assert_eq!(file.current(), Some('\n'));
        assert_eq!(file.get_current_indent_level(), 0);

        // 'c'
        file.next();
        assert_eq!(file.current(), Some('c'));
        assert_eq!(file.get_current_indent_level(), 0);

        // 'd'
        file.next();
        assert_eq!(file.current(), Some('d'));
        assert_eq!(file.get_current_indent_level(), 1);

        // '\n'
        file.next();
        assert_eq!(file.current(), Some('\n'));
        assert_eq!(file.get_current_indent_level(), 0);

        // 'e'
        file.next();
        assert_eq!(file.current(), Some('e'));
        assert_eq!(file.get_current_indent_level(), 0);

        // 'f'
        file.next();
        assert_eq!(file.current(), Some('f'));
        assert_eq!(file.get_current_indent_level(), 1);

        // None
        file.next();
        assert_eq!(file.current(), None);
    }

    #[test]
    fn test_file_eof_after_consumption() {
        let test_file = TestFile::new(b"xy");
        let mut file = File::new(&test_file.path).unwrap();
        // consume all bytes
        file.next(); // to 'y'
        file.next(); // to EOF
        assert_eq!(file.current(), None);
        assert!(!file.more());
    }

    #[test]
    fn test_file_next_safe_at_eof() {
        let test_file = TestFile::new(b"a");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.current(), Some('a'));
        file.next(); // move to EOF
        assert_eq!(file.current(), None);
        // calling next again should be safe and leave us at EOF
        file.next();
        assert_eq!(file.current(), None);
        assert!(!file.more());
    }

    #[test]
    fn test_file_reset_restores_after_eof() {
        let test_file = TestFile::new(b"- [Sammy Sosa, 63, 0.288]");
        let mut file = File::new(&test_file.path).unwrap();
        assert_eq!(file.current(), Some('-'));
        file.next(); // move to ' '
        file.next(); // move to '['
        assert_eq!(file.current(), Some('['));
        file.next(); // move to 'S'
        file.next(); // move to 'a'
        file.next(); // move to 'm'
        file.next(); // move to 'm'
        file.next(); // move to 'y'
        file.next(); // move to ' '
        file.next(); // move to 'S'
        file.next(); // move to 'o'
        file.next(); // move to 's'
        file.next(); // move to 'a'
        file.next(); // move to ','
        file.next(); // move to ' '
        file.next(); // move to '6'
        file.next(); // move to '3'
        file.next(); // move to ','
        file.next(); // move to ' '
        file.next(); // move to '0'
        file.next(); // move to '.'
        file.next(); // move to '2'
        file.next(); // move to '8'
        file.next(); // move to '8'
        file.next(); // move to ']'
        file.next(); // move to EOF
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
}
