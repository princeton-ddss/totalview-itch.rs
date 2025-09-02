use std::io::{Read, Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};
use getset::Getters;

use super::{read_kind, read_nanoseconds, read_price, read_ticker};
use super::{Context, ReadMessage, Version};
use super::{IntoNOIIMessage, NOIIMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct NetOrderImbalanceIndicator {
    nanoseconds: u64,
    kind: char,
    paired_shares: u64,
    imbalance_shares: u64,
    imbalance_direction: char,
    ticker: String,
    far_price: u32,
    near_price: u32,
    current_reference_price: u32,
    cross_type: char,
    price_variation_indicator: char,
}

impl ReadMessage for NetOrderImbalanceIndicator {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let paired_shares = buffer.read_u64::<NetworkEndian>()?;
        let imbalance_shares = buffer.read_u64::<NetworkEndian>()?;
        let imbalance_direction = buffer.read_u8().map(char::from)?;
        let ticker = read_ticker(buffer)?;
        let far_price = read_price(buffer)?;
        let near_price = read_price(buffer)?;
        let current_reference_price = read_price(buffer)?;
        let cross_type = buffer.read_u8().map(char::from)?;
        let price_variation_indicator = buffer.read_u8().map(char::from)?;

        Ok(Self {
            nanoseconds,
            kind,
            paired_shares,
            imbalance_shares,
            imbalance_direction,
            ticker,
            far_price,
            near_price,
            current_reference_price,
            cross_type,
            price_variation_indicator,
        })
    }
}

impl IntoNOIIMessage for NetOrderImbalanceIndicator {
    fn into_noii_message(self, date: String) -> NOIIMessage {
        NOIIMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            ticker: self.ticker,
            paired_shares: self.paired_shares,
            imbalance_shares: self.imbalance_shares,
            imbalance_direction: self.imbalance_direction,
            far_price: self.far_price,
            near_price: self.near_price,
            ref_price: self.current_reference_price,
            cross_type: self.cross_type,
            var_indicator: self.price_variation_indicator,
        }
    }
}
