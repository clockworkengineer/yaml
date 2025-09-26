use crate::io::traits::ISource;

/// A memory buffer implementation for reading JSON data from bytes.
/// Provides functionality to traverse and read byte content from memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    buffer: Vec<u8>,
    /// Current reading position in the buffer
    position: usize,
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
        Self { buffer : to_add.to_vec(), position: 0 }
    }
    /// Converts the buffer content to a String.
    ///
    /// # Returns
    /// A String containing UTF-8 interpretation of the buffer bytes.
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }
}

impl ISource for Buffer {
    /// Moves to the next character in the buffer
    fn next(&mut self) {
        self.position += 1;
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
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_source_buffer_works() {
        let  source = Buffer::new(String::from("i32e").as_bytes());
        assert_eq!(source.to_string(), "i32e");
    }
    #[test]
    fn read_character_from_source_buffer_works() {
        let  mut source = Buffer::new(String::from("i32e").as_bytes());
        match source.current() { Some('i') => assert!(true), _ => assert!(false)}
    }
    #[test]
    fn move_to_next_character_in_source_buffer_works() {
        let  mut source = Buffer::new(String::from("i32e").as_bytes());
        source.next();
        match source.current() { Some('3') => assert!(true), _ => assert!(false)}
    }
    #[test]
    fn move_to_last_character_in_source_buffer_works() {
        let  mut source = Buffer::new(String::from("i32e").as_bytes());
        while source.more() { source.next()}
        match source.current() { None => assert!(true), _ => assert!(false)}
    }
    #[test]
    fn reset_in_source_buffer_works() {
        let  mut source = Buffer::new(String::from("i32e").as_bytes());
        while source.more() { source.next()}
        source.reset();
        match source.current() { Some('i') => assert!(true), _ => assert!(false)}
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
            _ => assert!(false)
        }
    }

}