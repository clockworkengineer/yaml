//! Shared IO helpers for buffer and file sources/destinations
use std::io::{Read, Write, Result};

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
