use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_kind, read_nanoseconds, read_refno};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct DeleteOrder {
    nanoseconds: u64,
    kind: char,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    from_replace: bool,
}

impl DeleteOrder {
    pub(crate) fn new(
        nanoseconds: u64,
        kind: char,
        ticker: String,
        side: Side,
        price: u32,
        shares: u32,
        refno: u64,
        from_replace: bool,
    ) -> Self {
        Self {
            nanoseconds,
            kind,
            ticker,
            side,
            price,
            shares,
            refno,
            from_replace,
        }
    }
}

impl ReadMessage for DeleteOrder {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        // Read data from buffer
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
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
            kind,
            ticker: order.ticker,
            side: order.side,
            price: order.price,
            shares: order.shares,
            refno,
            from_replace: false,
        })
    }
}

impl IntoOrderMessage for DeleteOrder {
    fn into_order_message(self, date: String) -> OrderMessage {
        OrderMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            ticker: self.ticker,
            side: self.side,
            price: self.price,
            shares: self.shares,
            refno: self.refno,
            from_replace: self.from_replace,
            mpid: None,
        }
    }
}
