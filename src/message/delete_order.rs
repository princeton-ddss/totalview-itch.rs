use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_nanoseconds, read_refno};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct DeleteOrder {
    nanoseconds: u64,
    refno: u64,
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
}

impl DeleteOrder {
    pub(crate) fn new(
        nanoseconds: u64,
        refno: u64,
        ticker: String,
        side: Side,
        price: u32,
        shares: u32,
    ) -> Self {
        Self {
            nanoseconds,
            refno,
            ticker,
            side,
            price,
            shares,
        }
    }
}

impl ReadMessage for DeleteOrder {
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

        // Update context
        let order = context
            .active_orders
            .remove(&refno)
            .expect("Order not found");

        // Return message
        Ok(Self {
            nanoseconds,
            refno,
            side: order.side,
            shares: order.shares,
            ticker: order.ticker,
            price: order.price,
        })
    }
}

impl IntoOrderMessage for DeleteOrder {
    fn into_order_message(self, date: String) -> OrderMessage {
        OrderMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: 'D',
            ticker: self.ticker,
            side: self.side,
            price: self.price,
            shares: self.shares,
            refno: self.refno,
            new_refno: None,
            mpid: None,
        }
    }
}
