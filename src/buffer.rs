use std::fs::File;
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::path::Path;

pub struct Buffer<const N: usize> {
    file: File,
    cursor: Cursor<[u8; N]>,
}

impl<const N: usize> Buffer<N> {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Result<Self> {
        let mut file = File::open(filepath)?;
        let mut inner = [0; N]; // Fixed-size "inner" buffer
        file.read(&mut inner)?; // Initial load
        let cursor = Cursor::new(inner);

        Ok(Self { file, cursor })
    }

    fn refill(&mut self) -> Result<()> {
        let pos = self.cursor.position() as usize;
        let inner = self.cursor.get_mut();

        // Move unread bytes to the front
        let n = N - pos; // Number of unread bytes
        for i in 0..n {
            inner[i] = inner[pos + i];
        }

        // Refill the rest
        self.file.read(&mut inner[n..])?;

        // Reset the position
        self.cursor.set_position(0);

        Ok(())
    }
}

impl<const N: usize> Read for Buffer<N> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() > N {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Cannot read data greater than the buffer size",
            ));
        }

        let pos = self.cursor.position() as usize;

        if pos + buf.len() > N {
            self.refill()?;
        }

        self.cursor.read(buf)
    }
}

impl<const N: usize> Seek for Buffer<N> {
    fn seek(&mut self, style: SeekFrom) -> Result<u64> {
        let pos = self.cursor.position();
        let target_pos = self.cursor.seek(style)?;

        if target_pos < N as u64 {
            Ok(target_pos)
        } else {
            let d = target_pos - pos; // Seek distance

            if d >= N as u64 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Seek distance cannot exceed the buffer size",
                ));
            }

            self.cursor.set_position(pos);

            self.refill()?;

            self.cursor.set_position(d);

            Ok(d)
        }
    }
}

impl<const N: usize> Peek for Buffer<N> {}

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
