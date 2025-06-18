use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_nanoseconds, read_price, read_refno, read_shares, read_side, read_ticker};
use super::{Context, OrderState, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct AddOrder {
    nanoseconds: u64,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    from_replace: bool,
}

impl AddOrder {
    pub(crate) fn new(
        nanoseconds: u64,
        ticker: String,
        side: Side,
        price: u32,
        shares: u32,
        refno: u64,
        from_replace: bool,
    ) -> Self {
        Self {
            nanoseconds,
            ticker,
            side,
            price,
            shares,
            refno,
            from_replace,
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
            ticker: ticker.clone(),
            side,
            price,
            shares,
        };
        context.active_orders.insert(refno, order);

        // Return message
        Ok(Self {
            nanoseconds,
            ticker,
            side,
            price,
            shares,
            refno,
            from_replace: false,
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
            from_replace: self.from_replace,
            mpid: None,
        }
    }
}
