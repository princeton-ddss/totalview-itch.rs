use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_kind, read_matchno, read_nanoseconds};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoTradeMessage, TradeMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct BrokenTrade {
    nanoseconds: u64,
    kind: char,
    matchno: u64,
}

impl ReadMessage for BrokenTrade {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let matchno = read_matchno(buffer)?;

        Ok(Self {
            nanoseconds,
            kind,
            matchno,
        })
    }
}

impl IntoTradeMessage for BrokenTrade {
    fn into_trade_message(self, date: String) -> TradeMessage {
        TradeMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            refno: 0,               // Broken trades don't have reference numbers
            side: Side::Buy,        // Broken trades don't have a specific side
            shares: 0,              // Broken trades don't have shares
            ticker: "".to_string(), // Broken trades don't specify ticker
            price: 0,               // Broken trades don't have price
            matchno: self.matchno,
            cross_price: 0,
            cross_type: ' ',
        }
    }
}
