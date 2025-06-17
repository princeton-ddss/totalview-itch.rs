mod add_order;
mod cancel_order;
mod delete_order;
mod execute_order;
mod replace_order;
mod system_event;

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Result, Seek};

use byteorder::{NetworkEndian, ReadBytesExt};
use getset::Getters;
use serde::Serialize;
use strum_macros::Display;

use crate::buffer::Peek;

pub use add_order::AddOrder;
pub use cancel_order::CancelOrder;
pub use delete_order::DeleteOrder;
pub use execute_order::ExecuteOrder;
pub use system_event::SystemEvent;

pub(crate) use replace_order::read_replace_order;

#[derive(Debug)]
pub enum Message {
    SystemEvent(SystemEvent),
    AddOrder(AddOrder),
    ExecuteOrder(ExecuteOrder),
    CancelOrder(CancelOrder),
    DeleteOrder(DeleteOrder),
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

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum Side {
    #[serde(rename = "B")]
    Buy,
    #[serde(rename = "S")]
    Sell,
}

struct OrderState {
    side: Side,
    shares: u32,
    ticker: String,
    price: u32,
}

pub(crate) struct Context {
    clock: Option<u32>, // Tracks number of seconds past midnight (applicable for Version 4.1)
    active_orders: HashMap<u64, OrderState>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            clock: None,
            active_orders: HashMap::new(),
        }
    }

    pub fn update_clock(&mut self, seconds: u32) {
        self.clock = Some(seconds);
    }

    pub fn has_order(&self, refno: u64) -> bool {
        self.active_orders.contains_key(&refno)
    }
}

pub(crate) trait ReadMessage: Sized {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek;
}

fn read_nanoseconds<T: Read>(buffer: &mut T, version: &Version, clock: Option<u32>) -> Result<u64> {
    match version {
        Version::V41 => {
            let seconds = clock.expect("Clock info missing");
            let nanoseconds = buffer.read_u32::<NetworkEndian>()?;
            Ok((seconds as u64) * 1_000_000_000 + (nanoseconds as u64))
        }
        Version::V50 => buffer.read_u48::<NetworkEndian>(),
    }
}

fn read_shares<T: Read>(buffer: &mut T) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_price<T: Read>(buffer: &mut T) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_refno<T: Read>(buffer: &mut T) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_side<T: Read>(buffer: &mut T) -> Result<Side> {
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

fn read_event_code<T: Read>(buffer: &mut T) -> Result<EventCode> {
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

fn read_ticker<T: Read>(buffer: &mut T) -> Result<String> {
    let mut buf = vec![0; 8];
    buffer.read_exact(&mut buf)?;
    match String::from_utf8(buf) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}

pub(crate) fn peek_ticker_ahead<T: Peek>(buffer: &mut T, ahead: usize) -> Result<String> {
    let buf = buffer.peek_ahead(ahead, 8)?;
    match String::from_utf8(buf) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}

pub(crate) fn peek_refno_ahead<T: Peek>(buffer: &mut T, ahead: usize) -> Result<u64> {
    let buf = buffer.peek_ahead(ahead, 8)?;
    let arr: [u8; 8] = buf.try_into().unwrap();
    Ok(u64::from_be_bytes(arr))
}

// Data schema for storing order-related messages
#[derive(Debug, Getters, Serialize)]
pub struct OrderMessage {
    #[getset(get = "pub")]
    date: String,
    nanoseconds: u64,
    kind: char,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    mpid: Option<String>,
}

pub trait IntoOrderMessage {
    fn into_order_message(self, date: String) -> OrderMessage;
}
