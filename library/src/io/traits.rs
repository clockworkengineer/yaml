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
    /// Resets are supported; prefer `save_state`/`restore_state` for speculative reads.

    /// Opaque, concrete snapshot of a source read position and metadata.
    /// Using a concrete struct avoids associated-type issues with dyn ISource.
    fn save_state(&mut self) -> SaveState;

    /// Restore a previously-saved state.
    fn restore_state(&mut self, state: SaveState);

    // (previous object-safe save/restore removed in favor of concrete SaveState)

    fn is_whitespace(&self, c: char) -> bool {
        c == ' ' || c == '\t'
    }

    fn get_current_indent_level(&self) -> usize;
}

/// Concrete save/restore snapshot used by all ISource implementations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveState {
    /// Absolute byte position in the underlying source (file cursor or buffer index).
    pub pos: u64,
    /// The current byte at that position when the snapshot was taken (if any).
    pub current_byte: Option<u8>,
    /// Column (indent) at the snapshot.
    pub column: usize,
    /// Line number at the snapshot.
    pub line: usize,
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
