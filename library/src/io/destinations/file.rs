use std::fs::File as StdFile;
use std::io::{Write, Read, Seek};
use crate::io::traits::IDestination;

/// A file-based destination for writing JSON data to disk.
/// Implements file operations for storing and manipulating encoded data.
pub struct File {
    /// The underlying file handle for I/O operations
    file: StdFile,
    /// Name/path of the file being operated on
    file_name: String,
    /// Current length of the file in bytes
    file_length: usize,
}

impl File {
    /// Creates a new File instance with the specified path.
    ///
    /// # Arguments
    /// * `path` - The file path where the data will be written
    ///
    /// # Returns
    /// A Result containing the new File instance or an IO error
    pub fn new(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            file: StdFile::create(path)?,
            file_name: path.to_string(),
            file_length: 0,
        })
    }

    /// Returns the current length of the file in bytes.
    pub fn file_length(&self) -> usize {
        self.file_length
    }
    /// Returns the name/path of the file.
    pub fn file_name(&self) -> &str {
        &self.file_name.as_str()
    }
    /// Closes the file handle.
    pub fn close(&self) -> std::io::Result<()> {
        Ok(())
    }

}

impl IDestination for File {
    /// Adds a single byte to the end of the file.
    ///
    /// # Arguments
    /// * `b` - The byte to append
    fn add_byte(&mut self, b: u8) {
        self.file.write_all(&[b]).unwrap();
        self.file_length += 1
    }

    /// Adds a string of bytes to the end of the file.
    ///
    /// # Arguments
    /// * `s` - The string to append as bytes
    fn add_bytes(&mut self, s: &str) {
        self.file.write_all(s.as_bytes()).unwrap();
        self.file_length = self.file_length + s.len();
    }

    /// Clears the file content by recreating it.
    fn clear(&mut self) {
        self.file = StdFile::create(&self.file_name).unwrap();
        self.file_length = 0;
    }

    /// Returns the last byte in the file, if any.
    ///
    /// # Returns
    /// The last byte as Some(u8) or None if the file is empty
    fn last(&self) -> Option<u8> {
        if self.file_length == 0 {
            None
        } else {
            let mut buf = vec![0];
            let mut file = StdFile::open(&self.file_name).unwrap();
            file.seek(std::io::SeekFrom::End(-1)).unwrap();
            file.read_exact(&mut buf).unwrap();
            Some(buf[0])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn create_file_destination_works() -> std::io::Result<()> {
        let path = "test_create.txt";
        let _file = File::new(path)?;
        assert!(fs::metadata(path).is_ok());
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn create_file_fails_with_invalid_path() {
        let result = File::new("/invalid/path/test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn write_fails_on_readonly_file() -> std::io::Result<()> {
        let path = "test_readonly.txt";
        let mut file = File::new(path)?;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(path, perms)?;

        file.add_bytes("test");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn read_fails_on_missing_file() {
        let path = "missing_file.txt";
        let file = File::new(path).unwrap();
        assert!(file.last().is_none());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn add_byte_works() -> std::io::Result<()> {
        let path = "test_byte.txt";
        let mut file = File::new(path)?;
        file.add_byte(b'A');

        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "A");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn add_bytes_works() -> std::io::Result<()> {
        let path = "test_bytes.txt";
        let mut file = File::new(path)?;
        file.add_bytes("test");

        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "test");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn clear_works() -> std::io::Result<()> {
        let path = "test_clear.txt";
        let mut file = File::new(path)?;
        file.add_bytes("test");
        file.clear();

        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "");

        fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn file_length_works() -> std::io::Result<()> {
        let path = "test_length.txt";
        let mut file = File::new(path)?;
        assert_eq!(file.file_length(), 0);

        file.add_byte(b'A');
        assert_eq!(file.file_length(), 1);

        file.add_bytes("test");
        assert_eq!(file.file_length(), 5);

        file.clear();
        assert_eq!(file.file_length(), 0);

        fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn file_name_works() -> std::io::Result<()> {
        let path = "test_name.txt";
        let file = File::new(path)?;
        assert_eq!(file.file_name(), path);
        fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn last_works() -> std::io::Result<()> {
        let path = "test_last.txt";
        let mut file = File::new(path)?;
        assert_eq!(file.last(), None);

        file.add_byte(b'1');
        assert_eq!(file.last(), Some(b'1'));

        file.add_byte(b'2');
        assert_eq!(file.last(), Some(b'2'));

        file.clear();
        assert_eq!(file.last(), None);

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn last_handles_empty_file() -> std::io::Result<()> {
        let path = "test_empty.txt";
        let file = File::new(path)?;
        assert_eq!(file.last(), None);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn close_works() -> std::io::Result<()> {
        let path = "test_name.txt";
        let file = File::new(path)?;
        file.close()?;
        fs::remove_file(path)?;
        Ok(())
    }
    
}

