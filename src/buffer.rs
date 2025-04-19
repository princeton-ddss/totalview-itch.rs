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
            let inner = self.cursor.get_mut();

            // Move unread bytes to the front
            let n = N - pos; // Number of unread bytes
            for i in 0..n {
                inner[i] = inner[pos + i];
            }

            // Refill the rest
            self.file.read(&mut inner[n..])?;

            // Complete reset
            self.cursor.set_position(0);
        }

        self.cursor.read(buf)
    }
}

impl<const N: usize> Seek for Buffer<N> {
    fn seek(&mut self, style: SeekFrom) -> Result<u64> {
        let target_pos = self.cursor.seek(style)?;

        if target_pos < N as u64 {
            Ok(target_pos)
        } else {
            let q = target_pos / N as u64; // Quotient
            let r = target_pos % N as u64; // Remainder

            // Load new data
            let inner = self.cursor.get_mut();
            for _ in 0..q {
                self.file.read(inner)?;
            }

            // Set new position
            self.cursor.set_position(r);

            Ok(r)
        }
    }
}
