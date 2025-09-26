use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom};


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

    /// Moves the reading position back one character
    pub fn backup(&mut self) {
        if let Ok(_) = self.file.seek(SeekFrom::Current(-2)) {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                Some(byte[0])
            } else {
                None
            };
        }
    }
}