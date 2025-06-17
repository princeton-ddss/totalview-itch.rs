use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_nanoseconds, read_price, read_refno, read_shares, read_side, read_ticker};
use super::{Context, OrderState, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct AddOrder {
    nanoseconds: u64,
    refno: u64,
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
}

impl AddOrder {
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

impl ReadMessage for AddOrder {
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
        let side = read_side(buffer)?;
        let shares = read_shares(buffer)?;
        let ticker = read_ticker(buffer)?;
        let price = read_price(buffer)?;

        // Update context
        let order = OrderState {
            side,
            shares,
            ticker: ticker.clone(),
            price,
        };
        context.active_orders.insert(refno, order);

        // Return message
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

impl IntoOrderMessage for AddOrder {
    fn into_order_message(self, date: String) -> OrderMessage {
        OrderMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: 'A',
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
