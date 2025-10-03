use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom};
use crate::io::traits::ISource;

pub struct File {
    file: StdFile,
    current_byte: Option<u8>,
    column: usize,
    line: usize,
}

impl File {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut file = StdFile::open(path)?;
        let mut current_byte = [0u8; 1];
        let has_byte = file.read(&mut current_byte)? == 1;

        Ok(Self {
            file,
            current_byte: if has_byte { Some(current_byte[0]) } else { None },
            column: 0,
            line: 0,
        })
    }
}

impl ISource for File {
    fn next(&mut self) {
        let mut byte = [0u8; 1];
        if self.current_byte.is_some() {
            self.column += 1;
            if self.current_byte.unwrap() == b'\n' {
                self.line += 1;
                self.column = 0;
            }
        }
        self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
            Some(byte[0])
        } else {
            None
        };
    }

    fn current(&mut self) -> Option<char> {
        self.current_byte.map(|b| b as char)
    }

    fn more(&mut self) -> bool {
        self.current_byte.is_some()
    }

    fn reset(&mut self) {
        if let Ok(_) = self.file.seek(SeekFrom::Start(0)) {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                Some(byte[0])
            } else {
                None
            };
            self.column = 0;
            self.line = 0;
        }
    }

    fn backup(&mut self) {
        if let Ok(_) = self.file.seek(SeekFrom::Current(-2)) {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                Some(byte[0])
            } else {
                None
            };
            if self.column > 0 {
                self.column -= 1;
            }
        }
    }

    fn get_current_indent_level(&self) -> usize {
        self.column
    }
}