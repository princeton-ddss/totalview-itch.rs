use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_matchno, read_nanoseconds, read_refno, read_shares};
use super::{ReadMessage, Version};

#[derive(Debug)]
pub struct ExecuteOrder {
    nanoseconds: u64,
    refno: u64,
    shares: u32,
    matchno: u64,
}

impl ReadMessage for ExecuteOrder {
    fn read<const N: usize>(buffer: &mut Buffer<N>, version: &Version) -> Result<Self> {
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }

        let nanoseconds = read_nanoseconds(buffer, version)?;
        let refno = read_refno(buffer, version)?;
        let shares = read_shares(buffer, version)?;
        let matchno = read_matchno(buffer, version)?;

        Ok(Self {
            nanoseconds,
            refno,
            shares,
            matchno,
        })
    }
}
