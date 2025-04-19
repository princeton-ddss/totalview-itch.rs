use std::io::{Error, ErrorKind, Read, Result};

use byteorder::{NetworkEndian, ReadBytesExt};
use strum_macros::Display;

use super::buffer::Buffer;

#[derive(Debug, PartialEq, Display)]
pub enum Version {
    #[strum(serialize = "Version 4.1")]
    V41,
    #[strum(serialize = "Version 5.0")]
    V50,
}

#[derive(Debug, PartialEq)]
pub enum EventCode {
    StartMessages,
    StartSystem,
    StartMarketHours,
    EndMarketHours,
    EndSystem,
    EndMessages,
    EmergencyMarketHalt,
    EmergencyMarketQuoteOnly,
    EmergencyMarketResumption,
}

#[derive(Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Timestamp {
    seconds: u32,
}

#[derive(Debug)]
pub struct SystemEvent {
    nanoseconds: u64,
    event_code: EventCode,
}

#[derive(Debug)]
pub struct AddOrder {
    nanoseconds: u64,
    refno: u64,
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
}

#[derive(Debug)]
pub struct ExecuteOrder {
    nanoseconds: u64,
    refno: u64,
    shares: u32,
    matchno: u64,
}

#[derive(Debug)]
pub struct CancelOrder {
    nanoseconds: u64,
    refno: u64,
    shares: u32,
}

#[derive(Debug)]
pub struct DeleteOrder {
    nanoseconds: u64,
    refno: u64,
}

#[derive(Debug)]
pub struct ReplaceOrder {
    nanoseconds: u64,
    refno: u64,
    new_refno: u64,
    shares: u32,
    price: u32,
}

#[derive(Debug)]
pub enum Message {
    Timestamp(Timestamp),
    SystemEvent(SystemEvent),
    AddOrder(AddOrder),
    ExecuteOrder(ExecuteOrder),
    CancelOrder(CancelOrder),
    DeleteOrder(DeleteOrder),
    ReplaceOrder(ReplaceOrder),
}

fn read_seconds(buffer: &mut Buffer, version: &Version) -> Result<u32> {
    match version {
        Version::V41 => buffer.read_u32::<NetworkEndian>(),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("{version} does not include <seconds> property"),
        )),
    }
}

fn read_nanoseconds(buffer: &mut Buffer, version: &Version) -> Result<u64> {
    match version {
        Version::V41 => buffer.read_u32::<NetworkEndian>().map(|n| n as u64),
        Version::V50 => buffer.read_u48::<NetworkEndian>(),
    }
}

fn read_shares(buffer: &mut Buffer, _version: &Version) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_price(buffer: &mut Buffer, _version: &Version) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_refno(buffer: &mut Buffer, _version: &Version) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_new_refno(buffer: &mut Buffer, _version: &Version) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_matchno(buffer: &mut Buffer, _version: &Version) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_side(buffer: &mut Buffer, _version: &Version) -> Result<Side> {
    let side = match buffer.read_u8().map(char::from)? {
        'B' => Side::Buy,
        'S' => Side::Sell,
        unknown_code => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid code for trading: {}", unknown_code),
            ));
        }
    };
    Ok(side)
}

fn read_event_code(buffer: &mut Buffer, _version: &Version) -> Result<EventCode> {
    let event_code = match buffer.read_u8().map(char::from)? {
        'O' => EventCode::StartMessages,
        'S' => EventCode::StartSystem,
        'Q' => EventCode::StartMarketHours,
        'M' => EventCode::EndMarketHours,
        'E' => EventCode::EndSystem,
        'C' => EventCode::EndMessages,
        'A' => EventCode::EmergencyMarketHalt,
        'R' => EventCode::EmergencyMarketQuoteOnly,
        'B' => EventCode::EmergencyMarketResumption,
        unknown_code => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid event code encountered: {}", unknown_code),
            ));
        }
    };
    Ok(event_code)
}

fn read_ticker(buffer: &mut Buffer, _version: &Version) -> Result<String> {
    let mut buf = vec![0; 8];
    buffer.read_exact(&mut buf)?;
    match String::from_utf8(buf) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}

pub trait ReadMessage: Sized {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self>;
}

impl ReadMessage for Timestamp {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let seconds = read_seconds(buffer, version)?;

        Ok(Self { seconds })
    }
}

impl ReadMessage for SystemEvent {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let nanoseconds = read_nanoseconds(buffer, version)?;
        let event_code = read_event_code(buffer, version)?;

        Ok(Self {
            nanoseconds,
            event_code,
        })
    }
}

impl ReadMessage for AddOrder {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let nanoseconds = read_nanoseconds(buffer, version)?;
        let refno = read_refno(buffer, version)?;
        let side = read_side(buffer, version)?;
        let shares = read_shares(buffer, version)?;
        let ticker = read_ticker(buffer, version)?;
        let price = read_price(buffer, version)?;

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

impl ReadMessage for ExecuteOrder {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let nanoseconds = read_nanoseconds(buffer, version)?;
        let refno = read_refno(buffer, version)?;
        let shares = read_shares(buffer, version)?;
        let matchno = read_matchno(buffer, version)?;

        Ok(Self {
            nanoseconds,
            refno,
            shares,
            matchno,
        })
    }
}

impl ReadMessage for CancelOrder {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let nanoseconds = read_nanoseconds(buffer, version)?;
        let refno = read_refno(buffer, version)?;
        let shares = read_shares(buffer, version)?;

        Ok(Self {
            nanoseconds,
            refno,
            shares,
        })
    }
}

impl ReadMessage for DeleteOrder {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let nanoseconds = read_nanoseconds(buffer, version)?;
        let refno = read_refno(buffer, version)?;

        Ok(Self { nanoseconds, refno })
    }
}

impl ReadMessage for ReplaceOrder {
    fn read(buffer: &mut Buffer, version: &Version) -> Result<Self> {
        let nanoseconds = read_nanoseconds(buffer, version)?;
        let refno = read_refno(buffer, version)?;
        let new_refno = read_new_refno(buffer, version)?;
        let shares = read_shares(buffer, version)?;
        let price = read_price(buffer, version)?;

        Ok(Self {
            nanoseconds,
            refno,
            new_refno,
            shares,
            price,
        })
    }
}
