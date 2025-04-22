use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_nanoseconds, read_refno};
use super::{ReadMessage, Version};

#[derive(Debug)]
pub struct DeleteOrder {
    nanoseconds: u64,
    refno: u64,
}

impl ReadMessage for DeleteOrder {
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

        Ok(Self { nanoseconds, refno })
    }
}
