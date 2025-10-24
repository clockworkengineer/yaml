
use std::fs::File;
use std::io::{Read, Result, Write};

/// Represents different Unicode text file formats with their corresponding byte order marks (BOM)
pub enum Format {
    Utf8,        // UTF-8 without BOM
    Utf8bom,     // UTF-8 with BOM (EF BB BF)
    Utf16le,     // UTF-16 Little Endian (FF FE)
    Utf16be,     // UTF-16 Big Endian (FE FF)
    Utf32le,     // UTF-32 Little Endian (FF FE 00 00)
    Utf32be,     // UTF-32 Big Endian (00 00 FE FF)
}

impl Format {
    /// Returns the byte order mark (BOM) bytes for each format
    fn get_bom(&self) -> &'static [u8] {
        match self {
            Format::Utf8 => &[],
            Format::Utf8bom => &[0xEF, 0xBB, 0xBF],
            Format::Utf16le => &[0xFF, 0xFE],
            Format::Utf16be => &[0xFE, 0xFF],
            Format::Utf32le => &[0xFF, 0xFE, 0x00, 0x00],
            Format::Utf32be => &[0x00, 0x00, 0xFE, 0xFF],
        }
    }
}

/// Detects the Unicode format of a text file by examining its byte order mark (BOM)
pub fn detect_format(filename: &str) -> Result<Format> {
    let mut file = File::open(filename)?;
    let mut bom_buffer = [0u8; 4];
    let bytes_read = file.read(&mut bom_buffer)?;

    let format = match &bom_buffer[..bytes_read] {
        [0xEF, 0xBB, 0xBF, ..] => Format::Utf8bom,
        [0xFE, 0xFF, ..] => Format::Utf16be,
        [0xFF, 0xFE, 0x00, 0x00] => Format::Utf32le,
        [0x00, 0x00, 0xFE, 0xFF] => Format::Utf32be,
        [0xFF, 0xFE, ..] => Format::Utf16le,
        _ => Format::Utf8
    };

    Ok(format)
}

/// Writes a string to a file in the specified Unicode format
pub fn write_file_from_string(filename: &str, content: &str, format: Format) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(format.get_bom())?;

    match format {
        Format::Utf8 | Format::Utf8bom => {
            file.write_all(content.as_bytes())?;
        }
        Format::Utf16le => {
            for c in content.encode_utf16() {
                file.write_all(&c.to_le_bytes())?;
            }
        }
        Format::Utf16be => {
            for c in content.encode_utf16() {
                file.write_all(&c.to_be_bytes())?;
            }
        }
        Format::Utf32le => {
            for c in content.chars() {
                file.write_all(&(c as u32).to_le_bytes())?;
            }
        }
        Format::Utf32be => {
            for c in content.chars() {
                file.write_all(&(c as u32).to_be_bytes())?;
            }
        }
    }
    Ok(())
}

/// Reads a text file and returns its content as a String, handling different Unicode formats
pub fn read_file_to_string(filename: &str) -> Result<String> {
    let mut content = String::new();
    let format = detect_format(filename)?;
    let mut file = File::open(filename)?;

    /// Helper function to read and skip over the BOM bytes
    fn read_and_skip_bom(file: &mut File, size: usize) -> Result<()> {
        let mut buf = vec![0u8; size];
        file.read_exact(&mut buf)
    }

    /// Helper function to process UTF-16 encoded files
    fn process_utf16(file: &mut File, is_be: bool) -> Result<String> {
        read_and_skip_bom(file, 2)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let content = String::from_utf16(
            &bytes.chunks(2)
                .map(|chunk| if is_be {
                    u16::from_be_bytes([chunk[0], chunk[1]])
                } else {
                    u16::from_le_bytes([chunk[0], chunk[1]])
                })
                .collect::<Vec<u16>>()
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(content.replace("\r\n", "\n"))
    }

    /// Helper function to process UTF-32 encoded files
    fn process_utf32(file: &mut File, is_be: bool) -> Result<String> {
        read_and_skip_bom(file, 4)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let content = bytes.chunks(4)
            .map(|chunk| if is_be {
                u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            } else {
                u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            })
            .map(|cp| char::from_u32(cp).unwrap_or('\u{FFFD}'))
            .collect::<String>();

        Ok(content.replace("\r\n", "\n"))
    }

    match format {
        Format::Utf8bom => {
            read_and_skip_bom(&mut file, 3)?;
            file.read_to_string(&mut content)?;
        }
        Format::Utf16be => return process_utf16(&mut file, true),
        Format::Utf16le => return process_utf16(&mut file, false),
        Format::Utf32be => return process_utf32(&mut file, true),
        Format::Utf32le => return process_utf32(&mut file, false),
        Format::Utf8 => {
            file.read_to_string(&mut content)?;
        }
    }

    Ok(content.replace("\r\n", "\n"))
}


