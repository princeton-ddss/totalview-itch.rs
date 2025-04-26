mod add_order;
mod cancel_order;
mod delete_order;
mod execute_order;
mod replace_order;
mod system_event;

use std::collections::HashMap;
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

#[derive(Debug)]
pub enum Message {
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

struct OrderState {
    ticker: String,
    price: u32,
    shares: u32,
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
    fn read<const N: usize>(
        buffer: &mut Buffer<N>,
        version: &Version,
        context: &mut Context,
    ) -> Result<Self>;
}

fn read_nanoseconds<const N: usize>(
    buffer: &mut Buffer<N>,
    version: &Version,
    clock: Option<u32>,
) -> Result<u64> {
    match version {
        Version::V41 => {
            let seconds = clock.expect("Clock info missing");
            let nanoseconds = buffer.read_u32::<NetworkEndian>()?;
            Ok((seconds as u64) * 1_000_000_000 + (nanoseconds as u64))
        }
        Version::V50 => buffer.read_u48::<NetworkEndian>(),
    }
}

fn read_shares<const N: usize>(buffer: &mut Buffer<N>) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

fn read_price<const N: usize>(buffer: &mut Buffer<N>) -> Result<u32> {
    buffer.read_u32::<NetworkEndian>()
}

pub(crate) fn read_refno<const N: usize>(buffer: &mut Buffer<N>) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_new_refno<const N: usize>(buffer: &mut Buffer<N>) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_matchno<const N: usize>(buffer: &mut Buffer<N>) -> Result<u64> {
    buffer.read_u64::<NetworkEndian>()
}

fn read_side<const N: usize>(buffer: &mut Buffer<N>) -> Result<Side> {
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

fn read_event_code<const N: usize>(buffer: &mut Buffer<N>) -> Result<EventCode> {
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

pub(crate) fn read_ticker<const N: usize>(buffer: &mut Buffer<N>) -> Result<String> {
    let mut buf = vec![0; 8];
    buffer.read_exact(&mut buf)?;
    match String::from_utf8(buf) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}
