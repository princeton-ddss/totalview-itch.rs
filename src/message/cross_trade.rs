use std::io::{Read, Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};
use getset::Getters;

use super::{read_kind, read_matchno, read_nanoseconds, read_price, read_ticker};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoTradeMessage, TradeMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct CrossTrade {
    nanoseconds: u64,
    kind: char,
    shares: u64,
    ticker: String,
    cross_price: u32,
    matchno: u64,
    cross_type: char,
}

impl ReadMessage for CrossTrade {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let shares = buffer.read_u64::<NetworkEndian>()?; // 8 bytes for cross trades
        let ticker = read_ticker(buffer)?;
        let cross_price = read_price(buffer)?;
        let matchno = read_matchno(buffer)?;
        let cross_type = buffer.read_u8().map(char::from)?;

        Ok(Self {
            nanoseconds,
            kind,
            shares,
            ticker,
            cross_price,
            matchno,
            cross_type,
        })
    }
}

impl IntoTradeMessage for CrossTrade {
    fn into_trade_message(self, date: String) -> TradeMessage {
        TradeMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            refno: 0,        // Cross trades don't have reference numbers
            side: Side::Buy, // Cross trades don't have a specific side
            shares: self.shares,
            ticker: self.ticker,
            price: self.cross_price,
            matchno: self.matchno,
            cross_price: self.cross_price,
            cross_type: self.cross_type,
        }
    }
}
