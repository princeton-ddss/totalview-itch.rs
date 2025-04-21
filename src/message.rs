mod add_order;
mod cancel_order;
mod delete_order;
mod execute_order;
mod replace_order;
mod system_event;
mod timestamp;

use std::io::{Error, ErrorKind, Read, Result};

use byteorder::{NetworkEndian, ReadBytesExt};
use strum_macros::Display;

use crate::buffer::Buffer;

pub use add_order::AddOrder;
pub use cancel_order::CancelOrder;
pub use delete_order::DeleteOrder;
pub use execute_order::ExecuteOrder;
pub use replace_order::ReplaceOrder;
pub use system_event::SystemEvent;
pub use timestamp::Timestamp;

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

pub trait ReadMessage: Sized {
    fn read<const N: usize>(buffer: &mut Buffer<N>, version: &Version) -> Result<Self>;
}

fn read_seconds<const N: usize>(buffer: &mut Buffer<N>, version: &Version) -> Result<u32> {
    if version != &Version::V41 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("{version} does not include <seconds> property"),
        ));
    }

    buffer.read_u32::<NetworkEndian>()
}

fn read_nanoseconds<const N: usize>(buffer: &mut Buffer<N>, version: &Version) -> Result<u64> {
    match version {
        Version::V41 => buffer.read_u32::<NetworkEndian>().map(|n| n as u64),
        Version::V50 => buffer.read_u48::<NetworkEndian>(),
    }
}

fn read_shares<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_price<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_refno<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_new_refno<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_matchno<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_side<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<Side> {
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

fn read_event_code<const N: usize>(
    buffer: &mut Buffer<N>,
    _version: &Version,
) -> Result<EventCode> {
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

pub fn read_ticker<const N: usize>(buffer: &mut Buffer<N>, _version: &Version) -> Result<String> {
    let mut buf = vec![0; 8];
    buffer.read_exact(&mut buf)?;
    match String::from_utf8(buf) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}
