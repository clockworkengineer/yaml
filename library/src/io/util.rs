//! Shared I/O Utilities
//!
//! Provides helper functions for reading and writing bytes and strings to and from
//! buffer and file sources/destinations. Used throughout the YAML I/O modules.
//!
//! Copyright (c) 2026 YAML Library Developers

use std::io::{Read, Result, Write};

/// Read all bytes from a reader into a Vec<u8>
pub fn read_all<R: Read>(mut reader: R) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Write all bytes from a slice to a writer
pub fn write_all<W: Write>(mut writer: W, data: &[u8]) -> Result<()> {
    writer.write_all(data)
}

/// Write all bytes from a string to a writer
pub fn write_str<W: Write>(mut writer: W, data: &str) -> Result<()> {
    writer.write_all(data.as_bytes())
}
