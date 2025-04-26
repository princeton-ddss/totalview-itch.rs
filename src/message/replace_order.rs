use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_nanoseconds, read_new_refno, read_price, read_refno, read_shares};
use super::{Context, ReadMessage, Version};

#[derive(Debug)]
pub struct ReplaceOrder {
    nanoseconds: u64,
    refno: u64,
    new_refno: u64,
    shares: u32,
    price: u32,
}

impl ReadMessage for ReplaceOrder {
    fn read<const N: usize>(buffer: &mut Buffer<N>, context: &Context) -> Result<Self> {
        if context.version == Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }

        let nanoseconds = read_nanoseconds(buffer, &context.version, context.clock)?;
        let refno = read_refno(buffer)?;
        let new_refno = read_new_refno(buffer)?;
        let shares = read_shares(buffer)?;
        let price = read_price(buffer)?;

        Ok(Self {
            nanoseconds,
            refno,
            new_refno,
            shares,
            price,
        })
    }
}
