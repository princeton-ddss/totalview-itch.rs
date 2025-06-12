use std::io::{Read, Result, Seek, SeekFrom};

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::buffer::Peek;
use crate::message::{peek_refno_ahead, peek_ticker_ahead, Context, Message, ReadMessage, Version};
use crate::message::{AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, ReplaceOrder, SystemEvent};

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
        T: Read + Seek + Peek,
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
                'S' => self.parse_system_event(buffer)?,
                'A' => self.parse_add_order(buffer)?,
                'E' => self.parse_execute_order(buffer)?,
                'X' => self.parse_cancel_order(buffer)?,
                'D' => self.parse_delete_order(buffer)?,
                'U' => self.parse_replace_order(buffer)?,
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

    fn parse_system_event<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek,
    {
        let data = SystemEvent::read(buffer, &self.version, &mut self.context)?;
        Ok(Some(Message::SystemEvent(data)))
    }

    fn parse_add_order<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let ticker = match self.version {
            Version::V41 => peek_ticker_ahead(buffer, 17)?,
            Version::V50 => peek_ticker_ahead(buffer, 23)?,
        };
        if self.tickers.contains(&ticker) {
            let data = AddOrder::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::AddOrder(data)))
        } else {
            Ok(None)
        }
    }

    fn parse_execute_order<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let refno = match self.version {
            Version::V41 => peek_refno_ahead(buffer, 4)?,
            Version::V50 => peek_refno_ahead(buffer, 10)?,
        };
        if self.context.has_order(refno) {
            let data = ExecuteOrder::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::ExecuteOrder(data)))
        } else {
            Ok(None)
        }
    }

    fn parse_cancel_order<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let refno = match self.version {
            Version::V41 => peek_refno_ahead(buffer, 4)?,
            Version::V50 => peek_refno_ahead(buffer, 10)?,
        };
        if self.context.has_order(refno) {
            let data = CancelOrder::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::CancelOrder(data)))
        } else {
            Ok(None)
        }
    }

    fn parse_delete_order<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let refno = match self.version {
            Version::V41 => peek_refno_ahead(buffer, 4)?,
            Version::V50 => peek_refno_ahead(buffer, 10)?,
        };
        if self.context.has_order(refno) {
            let data = DeleteOrder::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::DeleteOrder(data)))
        } else {
            Ok(None)
        }
    }

    fn parse_replace_order<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let refno = match self.version {
            Version::V41 => peek_refno_ahead(buffer, 4)?,
            Version::V50 => peek_refno_ahead(buffer, 10)?,
        };
        if self.context.has_order(refno) {
            let data = ReplaceOrder::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::ReplaceOrder(data)))
        } else {
            Ok(None)
        }
    }
}
