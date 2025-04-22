use std::io::{Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};

use super::buffer::Buffer;
use super::message::{read_ticker, Message, ReadMessage, Version};
use super::message::{AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, ReplaceOrder, SystemEvent};

pub struct Parser {
    version: Version,
    tickers: Vec<String>,
    clock: Option<u32>, // Tracks number of seconds past midnight (applicable for Version 4.1)
}

impl Parser {
    pub fn new(version: Version, tickers: Vec<String>) -> Self {
        Self {
            version,
            tickers,
            clock: None,
        }
    }

    pub fn extract_message<const N: usize>(&mut self, buffer: &mut Buffer<N>) -> Result<Message> {
        loop {
            // TODO: Add logic to handle reaching EOF
            let size = buffer.read_u16::<NetworkEndian>()?;
            let kind = buffer.read_u8().map(char::from)?;

            let msg = match kind {
                'T' => {
                    let seconds = buffer.read_u32::<NetworkEndian>()?;
                    self.clock = Some(seconds);
                    continue;
                }
                'S' => {
                    let data = SystemEvent::read(buffer, &self.version, self.clock)?;
                    Some(Message::SystemEvent(data))
                }
                'A' => {
                    let ticker = match self.version {
                        Version::V41 => Self::glimpse_ticker_ahead(buffer, 17)?,
                        Version::V50 => Self::glimpse_ticker_ahead(buffer, 23)?,
                    };
                    if self.tickers.contains(&ticker) {
                        let data = AddOrder::read(buffer, &self.version, self.clock)?;
                        Some(Message::AddOrder(data))
                    } else {
                        None
                    }
                }
                'E' => {
                    let data = ExecuteOrder::read(buffer, &self.version, self.clock)?;
                    Some(Message::ExecuteOrder(data))
                }
                'X' => {
                    let data = CancelOrder::read(buffer, &self.version, self.clock)?;
                    Some(Message::CancelOrder(data))
                }
                'D' => {
                    let data = DeleteOrder::read(buffer, &self.version, self.clock)?;
                    Some(Message::DeleteOrder(data))
                }
                'U' => {
                    let data = ReplaceOrder::read(buffer, &self.version, self.clock)?;
                    Some(Message::ReplaceOrder(data))
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

    fn glimpse_ticker_ahead<const N: usize>(
        buffer: &mut Buffer<N>,
        ahead: usize,
    ) -> Result<String> {
        let ticker_size = 8;
        let offset = (ahead + ticker_size) as i64;

        buffer.seek(SeekFrom::Current(offset))?; // Look ahead enough for rollback to work properly
        buffer.seek(SeekFrom::Current(-(ticker_size as i64)))?;
        let ticker = read_ticker(buffer)?;
        buffer.seek(SeekFrom::Current(-offset))?; // Restore position in buffer

        Ok(ticker)
    }
}
