use std::io::{Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};

use super::buffer::Buffer;
use super::message::{read_refno, read_ticker, Context, Message, ReadMessage, Version};
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

    pub fn extract_message<const N: usize>(&mut self, buffer: &mut Buffer<N>) -> Result<Message> {
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
                        Version::V41 => Self::glimpse_ticker_ahead(buffer, 17)?,
                        Version::V50 => Self::glimpse_ticker_ahead(buffer, 23)?,
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
                        Version::V41 => Self::glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => Self::glimpse_refno_ahead(buffer, 10)?,
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
                        Version::V41 => Self::glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => Self::glimpse_refno_ahead(buffer, 10)?,
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
                        Version::V41 => Self::glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => Self::glimpse_refno_ahead(buffer, 10)?,
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
                        Version::V41 => Self::glimpse_refno_ahead(buffer, 4)?,
                        Version::V50 => Self::glimpse_refno_ahead(buffer, 10)?,
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

    fn glimpse_refno_ahead<const N: usize>(buffer: &mut Buffer<N>, ahead: usize) -> Result<u64> {
        let refno_size = 8;
        let offset = (ahead + refno_size) as i64;

        buffer.seek(SeekFrom::Current(offset))?; // Look ahead enough for rollback to work properly
        buffer.seek(SeekFrom::Current(-(refno_size as i64)))?;
        let refno = read_refno(buffer)?;
        buffer.seek(SeekFrom::Current(-offset))?; // Restore position in buffer

        Ok(refno)
    }
}
