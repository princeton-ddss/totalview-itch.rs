use std::io::{Read, Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};

use super::buffer::Glimpse;
use super::message::{
    glimpse_refno_ahead, glimpse_ticker_ahead, Context, Message, ReadMessage, Version,
};
use super::message::{AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, ReplaceOrder, SystemEvent};

pub struct Parser {
    version: Version,
    tickers: Vec<String>,
    context: Context,
}

impl Parser {
    pub fn new(version: Version, tickers: Vec<String>) -> Self {
        Self {
            version,
            tickers,
            context: Context::new(),
        }
    }

    pub fn extract_message<T>(&mut self, buffer: &mut T) -> Result<Message>
    where
        T: Read + Seek + Glimpse,
    {
        loop {
            // TODO: Add logic to handle reaching EOF
            let size = buffer.read_u16::<NetworkEndian>()?;
            let kind = buffer.read_u8().map(char::from)?;

            if kind == 'T' {
                let seconds = buffer.read_u32::<NetworkEndian>()?;
                self.context.update_clock(seconds);
                continue;
            }

            let msg = match kind {
                'S' => {
                    let data = SystemEvent::read(buffer, &self.version, &mut self.context)?;
                    Some(Message::SystemEvent(data))
                }
                'A' => {
                    let ticker = match self.version {
                        Version::V41 => glimpse_ticker_ahead(buffer, 17)?,
                        Version::V50 => glimpse_ticker_ahead(buffer, 23)?,
                    };
                    if self.tickers.contains(&ticker) {
                        let data = AddOrder::read(buffer, &self.version, &mut self.context)?;
                        Some(Message::AddOrder(data))
                    } else {
                        None
                    }
                }
                'E' => {
                    let refno = match self.version {
                        Version::V41 => glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => glimpse_refno_ahead(buffer, 10)?,
                    };
                    if self.context.has_order(refno) {
                        let data = ExecuteOrder::read(buffer, &self.version, &mut self.context)?;
                        Some(Message::ExecuteOrder(data))
                    } else {
                        None
                    }
                }
                'X' => {
                    let refno = match self.version {
                        Version::V41 => glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => glimpse_refno_ahead(buffer, 10)?,
                    };
                    if self.context.has_order(refno) {
                        let data = CancelOrder::read(buffer, &self.version, &mut self.context)?;
                        Some(Message::CancelOrder(data))
                    } else {
                        None
                    }
                }
                'D' => {
                    let refno = match self.version {
                        Version::V41 => glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => glimpse_refno_ahead(buffer, 10)?,
                    };
                    if self.context.has_order(refno) {
                        let data = DeleteOrder::read(buffer, &self.version, &mut self.context)?;
                        Some(Message::DeleteOrder(data))
                    } else {
                        None
                    }
                }
                'U' => {
                    let refno = match self.version {
                        Version::V41 => glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => glimpse_refno_ahead(buffer, 10)?,
                    };
                    if self.context.has_order(refno) {
                        let data = ReplaceOrder::read(buffer, &self.version, &mut self.context)?;
                        Some(Message::ReplaceOrder(data))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            match msg {
                Some(m) => return Ok(m),
                None => {
                    let offset = (size - 1) as i64; // Message type has already been read
                    buffer.seek(SeekFrom::Current(offset))?;
                    continue;
                }
            }
        }
    }
}
