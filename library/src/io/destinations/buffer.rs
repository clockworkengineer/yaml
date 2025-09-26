use crate::io::traits::IDestination;
/// A memory buffer implementation for storing encoded JSON data as bytes.
/// Provides functionality to write and manipulate byte content in memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    pub buffer: Vec<u8>,
}

impl Buffer {
    /// Creates a new empty Buffer instance.
    ///
    /// # Returns
    /// A new Buffer with an empty internal byte vector.
    pub fn new() -> Self {
        Self { buffer: vec![] }
    }

    /// Converts the buffer content to a String.
    ///
    /// # Returns
    /// A String containing UTF-8 interpretation of the buffer bytes.
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }

}

impl IDestination for Buffer {
    /// Adds a single byte to the end of the buffer.
    fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    /// Adds multiple bytes from a string slice to the buffer.
    fn add_bytes(&mut self, bytes: &str) {
        self.buffer.extend_from_slice(bytes.as_bytes());
    }

    /// Clears all content from the buffer.
    fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the last byte in the buffer, if any.
    fn last(&self) -> Option<u8> {
        self.buffer.last().copied()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_creates_empty_buffer() {
        let buffer = Buffer::new();
        assert!(buffer.buffer.is_empty());
    }
    #[test]
    fn add_byte_to_destination_buffer_works() {
        let mut destination = Buffer::new();
        destination.add_byte(b'i');
        destination.add_byte(b'3');
        destination.add_byte(b'2');
        destination.add_byte(b'e');
        assert_eq!(destination.to_string(), "i32e");
    }
    #[test]
    fn add_bytes_to_destination_buffer_works() {
        let mut destination = Buffer::new();
        destination.add_bytes("i3");
        assert_eq!(destination.to_string(), "i3");
        destination.add_bytes("2e");
        assert_eq!(destination.to_string(), "i32e");
    }
    #[test]
    fn clear_destination_buffer_works() {
        let mut destination = Buffer::new();
        destination.add_bytes("i32e");
        assert_eq!(destination.to_string(), "i32e");
        destination.clear();
        assert_eq!(destination.to_string(), "");
    }
    #[test]
    fn last_works() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.last(), None);
        buffer.add_byte(b'1');
        assert_eq!(buffer.last(), Some(b'1'));
        buffer.add_byte(b'2');
        assert_eq!(buffer.last(), Some(b'2'));
        buffer.clear();
        assert_eq!(buffer.last(), None);
    }
    #[test]
    fn to_string_handles_non_utf8() {
        let mut buffer = Buffer::new();
        buffer.add_byte(0xFF);
        assert_eq!(buffer.to_string(), "�");
    }
    
}