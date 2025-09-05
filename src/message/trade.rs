use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{
    read_kind, read_matchno, read_nanoseconds, read_price, read_refno, read_shares, read_side,
    read_ticker,
};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoTradeMessage, TradeMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct Trade {
    nanoseconds: u64,
    kind: char,
    refno: u64,
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
    matchno: u64,
}

impl ReadMessage for Trade {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let refno = read_refno(buffer)?;
        let side = read_side(buffer)?;
        let shares = read_shares(buffer)?;
        let ticker = read_ticker(buffer)?;
        let price = read_price(buffer)?;
        let matchno = read_matchno(buffer)?;

        Ok(Self {
            nanoseconds,
            kind,
            refno,
            side,
            shares,
            ticker,
            price,
            matchno,
        })
    }
}

impl IntoTradeMessage for Trade {
    fn into_trade_message(self, date: String) -> TradeMessage {
        TradeMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            refno: self.refno,
            side: self.side,
            shares: self.shares as u64,
            ticker: self.ticker,
            price: self.price,
            matchno: self.matchno,
            cross_price: 0,  // Not applicable for regular trades
            cross_type: ' ', // Not applicable for regular trades
        }
    }
}
