use std::io::{Read, Result, Seek, SeekFrom};

use super::{read_nanoseconds, read_new_refno, read_price, read_refno, read_shares};
use super::{Context, ReadMessage, Version};

#[derive(Debug)]
pub struct ReplaceOrder {
    nanoseconds: u64,
    refno: u64,
    new_refno: u64,
    shares: u32,
    ticker: String,
    price: u32,
}

impl ReadMessage for ReplaceOrder {
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
        let new_refno = read_new_refno(buffer)?;
        let shares = read_shares(buffer)?;
        let price = read_price(buffer)?;

        // Update context
        let mut order = context
            .active_orders
            .remove(&refno)
            .expect("Order not found");
        let ticker = order.ticker.clone(); // For use after ownership move
        order.price = price;
        order.shares = shares;
        context.active_orders.insert(new_refno, order);

        // Return message
        Ok(Self {
            nanoseconds,
            refno,
            new_refno,
            shares,
            ticker,
            price,
        })
    }
}
