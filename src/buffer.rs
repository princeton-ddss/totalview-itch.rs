use std::fs::File;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};
use std::path::Path;

pub struct BufFile {
    reader: BufReader<File>,
}

impl BufFile {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Result<Self> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        Ok(Self { reader })
    }

    pub fn with_capacity<P: AsRef<Path>>(capacity: usize, filepath: P) -> Result<Self> {
        let file = File::open(filepath)?;
        let reader = BufReader::with_capacity(capacity, file);
        Ok(Self { reader })
    }
}

impl Read for BufFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}

impl Seek for BufFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.reader.seek(pos)
    }
}

impl Peek for BufFile {}

pub trait Peek: Read + Seek {
    fn peek_ahead(&mut self, ahead: usize, size: usize) -> Result<Vec<u8>> {
        let offset = (ahead + size) as i64;
        let mut buf = vec![0; size];

        self.seek(SeekFrom::Current(offset))?; // Look ahead enough for rollback to work properly
        self.seek(SeekFrom::Current(-(size as i64)))?;
        self.read_exact(&mut buf)?;
        self.seek(SeekFrom::Current(-offset))?; // Restore position

        Ok(buf)
    }
}
