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
        let mut current_byte = [0u8; 1];
        let has_byte = file.read(&mut current_byte)? == 1;

        Ok(Self {
            file,
            current_byte: if has_byte {
                Some(current_byte[0])
            } else {
                None
            },
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
                self.file.read(&mut byte2).unwrap_or(0);
                // Treat \r\n as a single \n
                if byte1[0] == b'\r' && byte2[0] == b'\n' {
                    self.line += 1;
                    self.column = 0;
                    self.current_byte = Some(b'\n');
                } else {
                    self.current_byte = Some(byte1[0]);
                    self.file.seek(SeekFrom::Current(-1)).unwrap();
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
        if let Ok(_) = self.file.seek(SeekFrom::Start(0)) {
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

    fn backup(&mut self) {
        if let Ok(_) = self.file.seek(SeekFrom::Current(-2)) {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                if byte[0] == b'\r' {
                    // If we see a '\r', try to seek back one more to check for '\n'
                    if let Ok(_) = self.file.seek(SeekFrom::Current(-2)) {
                        if self.file.read(&mut byte).unwrap_or(0) == 1 {}
                    }
                }
                Some(byte[0])
            } else {
                None
            };
            if self.column > 0 {
                self.column -= 1;
            }
        }
    }

    fn get_current_indent_level(&self) -> usize {
        self.column
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use super::*;
    use crate::nodes::node::{Node, QuoteType, Numeric};
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
    fn test_file_backup() {
        let test_file = TestFile::new(b"123\r\n456");
        let mut file = File::new(&test_file.path).unwrap();
        file.next(); // move to '2'
        file.next(); // move to '3'
        assert_eq!(file.current(), Some('3'));
        file.backup(); // should go back to '2'
        assert_eq!(file.current(), Some('2'));
        file.backup(); // should go back to '1'
        assert_eq!(file.current(), Some('1'));
        file.next(); // move to '2'
        file.next(); // move to '3'
        file.next(); // move to '\n'
        assert_eq!(file.current(), Some('\n'));
        file.next(); // move to '4'
        assert_eq!(file.current(), Some('4'));
        file.backup(); // should go back to '\n'
        assert_eq!(file.current(), Some('\n'));
        file.backup(); // should go back to '3'
        assert_eq!(file.current(), Some('3'));
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
            Node::Documents(vec![Node::Document(vec![Node::Array(vec![
                Node::Str("Sammy Sosa".to_string(), QuoteType::Unquoted),
                Node::Number(Numeric::Integer(63)),
                Node::Number(Numeric::Float(0.288))
            ])])])
        );
    }
}
