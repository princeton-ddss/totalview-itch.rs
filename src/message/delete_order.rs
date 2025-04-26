use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_nanoseconds, read_refno};
use super::{Context, ReadMessage, Version};

#[derive(Debug)]
pub struct DeleteOrder {
    nanoseconds: u64,
    refno: u64,
    shares: u32,
    ticker: String,
    price: u32,
}

impl ReadMessage for DeleteOrder {
    fn read<const N: usize>(
        buffer: &mut Buffer<N>,
        version: &Version,
        context: &mut Context,
    ) -> Result<Self> {
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }

        // Read data from buffer
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let refno = read_refno(buffer)?;

        // Update context
        let order = context
            .active_orders
            .remove(&refno)
            .expect("Order not found");

        // Return message
        Ok(Self {
            nanoseconds,
            refno,
            shares: order.shares,
            ticker: order.ticker,
            price: order.price,
        })
    }
}
