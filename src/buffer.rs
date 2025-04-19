use std::io::{Cursor, Read, Result, Seek, SeekFrom};
use std::path::Path;

pub struct Buffer {
    cursor: Cursor<Vec<u8>>,
}

impl Seek for Buffer {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.cursor.seek(pos)
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.cursor.read(buf)
    }
}

impl Buffer {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Self {
        // NOTE: The current approach loads the entire file content into memory
        // TODO: Read and process the file content in smaller chunks
        let data = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(data);

        Self { cursor }
    }

    pub fn skip(&mut self, size: u16) -> Result<()> {
        let offset = (size - 1) as i64;
        self.seek(SeekFrom::Current(offset))?;

        Ok(())
    }
}
