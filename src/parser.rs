use std::io::{Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};

use super::buffer::Buffer;
use super::message::{read_ticker, Context, Message, ReadMessage, Version};
use super::message::{AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, ReplaceOrder, SystemEvent};

pub struct Parser {
    tickers: Vec<String>,
    context: Context,
}

impl Parser {
    pub fn new(version: Version, tickers: Vec<String>) -> Self {
        let context = Context {
            version,
            clock: None,
        };

        Self { tickers, context }
    }

    pub fn extract_message<const N: usize>(&mut self, buffer: &mut Buffer<N>) -> Result<Message> {
        loop {
            // TODO: Add logic to handle reaching EOF
            let size = buffer.read_u16::<NetworkEndian>()?;
            let kind = buffer.read_u8().map(char::from)?;

            if kind == 'T' {
                let seconds = buffer.read_u32::<NetworkEndian>()?;
                self.context.clock = Some(seconds);
                continue;
            }

            let msg = match kind {
                'S' => {
                    let data = SystemEvent::read(buffer, &self.context)?;
                    Some(Message::SystemEvent(data))
                }
                'A' => {
                    let ticker = match self.context.version {
                        Version::V41 => Self::glimpse_ticker_ahead(buffer, 17)?,
                        Version::V50 => Self::glimpse_ticker_ahead(buffer, 23)?,
                    };
                    if self.tickers.contains(&ticker) {
                        let data = AddOrder::read(buffer, &self.context)?;
                        Some(Message::AddOrder(data))
                    } else {
                        None
                    }
                }
                'E' => {
                    let data = ExecuteOrder::read(buffer, &self.context)?;
                    Some(Message::ExecuteOrder(data))
                }
                'X' => {
                    let data = CancelOrder::read(buffer, &self.context)?;
                    Some(Message::CancelOrder(data))
                }
                'D' => {
                    let data = DeleteOrder::read(buffer, &self.context)?;
                    Some(Message::DeleteOrder(data))
                }
                'U' => {
                    let data = ReplaceOrder::read(buffer, &self.context)?;
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
