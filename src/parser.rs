use std::io::Result;

use byteorder::{NetworkEndian, ReadBytesExt};

use super::buffer::Buffer;
use super::message::{read_ticker, Message, ReadMessage, Version};
use super::message::{
    AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, ReplaceOrder, SystemEvent, Timestamp,
};

pub struct Parser {
    version: Version,
    tickers: Vec<String>,
}

impl Parser {
    pub fn new(version: Version, tickers: Vec<String>) -> Self {
        Self { version, tickers }
    }

    pub fn extract_message(&self, buffer: &mut Buffer) -> Result<Message> {
        loop {
            // TODO: Add logic to handle reaching the end of the buffer
            let size = buffer.read_u16::<NetworkEndian>()?;
            let kind = buffer.read_u8().map(char::from)?;

            let msg = match kind {
                'T' => {
                    let data = Timestamp::read(buffer, &self.version)?;
                    Some(Message::Timestamp(data))
                }
                'S' => {
                    let data = SystemEvent::read(buffer, &self.version)?;
                    Some(Message::SystemEvent(data))
                }
                'A' => {
                    let ticker = match self.version {
                        Version::V41 => self.glimpse_ticker_ahead(buffer, 17)?,
                        Version::V50 => self.glimpse_ticker_ahead(buffer, 23)?,
                    };
                    if self.tickers.contains(&ticker) {
                        let data = AddOrder::read(buffer, &self.version)?;
                        Some(Message::AddOrder(data))
                    } else {
                        None
                    }
                }
                'E' => {
                    let data = ExecuteOrder::read(buffer, &self.version)?;
                    Some(Message::ExecuteOrder(data))
                }
                'X' => {
                    let data = CancelOrder::read(buffer, &self.version)?;
                    Some(Message::CancelOrder(data))
                }
                'D' => {
                    let data = DeleteOrder::read(buffer, &self.version)?;
                    Some(Message::DeleteOrder(data))
                }
                'U' => {
                    let data = ReplaceOrder::read(buffer, &self.version)?;
                    Some(Message::ReplaceOrder(data))
                }
                _ => None,
            };

            match msg {
                Some(m) => return Ok(m),
                None => {
                    buffer.skip(size - 1)?; // Message type has already been read
                    continue;
                }
            }
        }
    }

    fn glimpse_ticker_ahead(&self, buffer: &mut Buffer, ahead: u16) -> Result<String> {
        let pos = buffer.get_position();
        buffer.skip(ahead)?;
        let ticker = read_ticker(buffer, &self.version)?;
        buffer.set_position(pos); // Restore position in buffer

        Ok(ticker)
    }
}
