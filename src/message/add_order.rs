use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_nanoseconds, read_price, read_refno, read_shares, read_side, read_ticker};
use super::{ReadMessage, Side, Version};

#[derive(Debug)]
pub struct AddOrder {
    nanoseconds: u64,
    refno: u64,
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
}

impl ReadMessage for AddOrder {
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
        let side = read_side(buffer)?;
        let shares = read_shares(buffer)?;
        let ticker = read_ticker(buffer)?;
        let price = read_price(buffer)?;

        Ok(Self {
            nanoseconds,
            refno,
            side,
            shares,
            ticker,
            price,
        })
    }
}
