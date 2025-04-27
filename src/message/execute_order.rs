use std::io::{Read, Result, Seek, SeekFrom};

use super::{read_matchno, read_nanoseconds, read_refno, read_shares};
use super::{Context, ReadMessage, Side, Version};

#[derive(Debug)]
pub struct ExecuteOrder {
    nanoseconds: u64,
    refno: u64,
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
    matchno: u64,
}

impl ReadMessage for ExecuteOrder {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }

        // Read data from buffer
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let refno = read_refno(buffer)?;
        let shares = read_shares(buffer)?;
        let matchno = read_matchno(buffer)?;

        // Update context
        let order = context
            .active_orders
            .get_mut(&refno)
            .expect("Order not found");
        order.shares -= shares;

        // Return message
        Ok(Self {
            nanoseconds,
            refno,
            side: order.side,
            shares,
            ticker: order.ticker.clone(),
            price: order.price,
            matchno,
        })
    }
}
