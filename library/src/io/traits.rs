/// Trait defining the interface for reading and traversing YAML data from a source.
/// Provides basic operations for sequential character-based reading.
pub trait ISource {
    /// Advances the reading position to the next character.
    fn next(&mut self);
    /// Returns the character at the current reading position.
    fn current(&mut self) -> Option<char>;
    /// Checks if there are more characters available to read.
    fn more(&mut self) -> bool;
    /// Resets the reading position to the beginning of the source.
    fn reset(&mut self);
    /// Moves the reading position back one character.
    fn backup(&mut self);

    fn is_whitespace(&self, c: char) -> bool {
        c == ' ' || c == '\t' || c == '\n' || c == '\r'
    }
}

/// Trait defining the interface for writing YAML data to a destination.
/// Provides operations for writing and managing byte content.
pub trait IDestination {
    /// Adds a single byte to the destination.
    fn add_byte(&mut self, byte: u8);
    /// Adds multiple bytes from a string slice to the destination.
    fn add_bytes(&mut self, bytes: &str);
    /// Clears all content from the destination.
    fn clear(&mut self);
    /// Returns the last byte in the destination, if any.
    fn last(&self) -> Option<u8>;
    
}