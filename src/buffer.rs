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

/// A trait for peeking ahead in a readable stream without consuming the data.
///
/// This trait extends `Read` and `Seek` to provide the ability to look ahead
/// at data that will be read in the future, without advancing the current
/// position in the stream.
pub trait Peek: Read + Seek {
    fn peek(&mut self, ahead: usize, size: usize) -> Result<Vec<u8>> {
        let original_pos = self.stream_position()?;
        let result = (|| {
            self.seek(SeekFrom::Current(ahead as i64))?;
            let mut buf = vec![0; size];
            self.read_exact(&mut buf)?;

            Ok(buf)
        })();
        self.seek(SeekFrom::Start(original_pos))?;

        result
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn peek_a_bit() {
        let file = assert_fs::NamedTempFile::new("test.txt").unwrap();
        file.write_str("abcdefghijkl").unwrap(); // 12 bytes
        let mut buffile = BufFile::new(file.path()).unwrap();
        let res = buffile.peek(4, 4).unwrap();
        assert_eq!(res, b"efgh");
        assert_eq!(buffile.reader.stream_position().unwrap(), 0);
    }

    #[test]
    fn peek_too_far() {
        let file = assert_fs::NamedTempFile::new("test.txt").unwrap();
        file.write_str("abcd").unwrap(); // 4 bytes
        let mut buffile = BufFile::new(file.path()).unwrap();
        let result = buffile.peek(0, 8);
        assert!(result.is_err());
        assert_eq!(buffile.reader.stream_position().unwrap(), 0);
    }
}
