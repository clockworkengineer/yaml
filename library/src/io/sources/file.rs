use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom};
use crate::io::traits::ISource;

/// A file-based implementation for reading JSON data from disk.
/// Provides functionality to read and traverse file content byte by byte.
pub struct File {
    /// Internal file handle for reading operations
    file: StdFile,
    /// Current byte being read from the file
    current_byte: Option<u8>,
}

impl File {
    /// Creates a new File instance from the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read from
    ///
    /// # Returns
    /// A Result containing either the new File instance or an IO error
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut file = StdFile::open(path)?;
        let mut current_byte = [0u8; 1];
        let has_byte = file.read(&mut current_byte)? == 1;

        Ok(Self {
            file,
            current_byte: if has_byte { Some(current_byte[0]) } else { None },
        })
    }
}

impl ISource for File {
    /// Moves to the next byte in the file
    fn next(&mut self) {
        let mut byte = [0u8; 1];
        self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
            Some(byte[0])
        } else {
            None
        };
    }

    /// Returns the current byte as a character
    fn current(&mut self) -> Option<char> {
        self.current_byte.map(|b| b as char)
    }

    /// Checks if there are more bytes to read
    fn more(&mut self) -> bool {
        self.current_byte.is_some()
    }

    /// Resets the file position to the start
    fn reset(&mut self) {
        if let Ok(_) = self.file.seek(SeekFrom::Start(0)) {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                Some(byte[0])
            } else {
                None
            };
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn create_test_file(content: &str) -> String {
        let path = format!("test_{}.txt", rand::random::<u32>());
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    fn cleanup_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn create_source_file_works() {
        let path = create_test_file("i32e");
        let source = File::new(&path);
        assert!(source.is_ok());
        cleanup_file(&path);
    }

    #[test]
    fn create_source_non_existent_file_fails() {
        let result = File::new("non_existent_file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn create_source_empty_file_works() {
        let path = create_test_file("");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), None);
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn read_character_from_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), Some('i'));
        cleanup_file(&path);
    }

    #[test]
    fn move_to_next_character_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        source.next();
        assert_eq!(source.current(), Some('3'));
        cleanup_file(&path);
    }

    #[test]
    fn move_to_last_character_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        assert_eq!(source.current(), None);
        cleanup_file(&path);
    }

    #[test]
    fn reset_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        source.reset();
        assert_eq!(source.current(), Some('i'));
        cleanup_file(&path);
    }

    #[test]
    fn reset_with_seek_error_maintains_state() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        cleanup_file(&path); // Remove the file to cause seek error
        source.reset();
        assert_eq!(source.current(), Some('i'));
    }

    #[test]
    fn read_complete_file_content_matches() {
        let test_content = "i32e";
        let path = create_test_file(test_content);
        let mut source = File::new(&path).unwrap();
        let mut content = String::new();
        while source.more() {
            content.push(source.current().unwrap());
            source.next();
        }
        assert_eq!(content, test_content);
        cleanup_file(&path);
    }
}