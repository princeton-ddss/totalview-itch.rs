use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_nanoseconds, read_refno, read_shares};
use super::{ReadMessage, Version};

#[derive(Debug)]
pub struct CancelOrder {
    nanoseconds: u64,
    refno: u64,
    shares: u32,
}

impl ReadMessage for CancelOrder {
    fn read<const N: usize>(
        buffer: &mut Buffer<N>,
        version: &Version,
        clock: Option<u32>,
    ) -> Result<Self> {
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }

        let nanoseconds = read_nanoseconds(buffer, version, clock)?;
        let refno = read_refno(buffer)?;
        let shares = read_shares(buffer)?;

        Ok(Self {
            nanoseconds,
            refno,
            shares,
        })
    }
}
