use std::io::{Cursor, Read, Result, Seek, SeekFrom};
use std::path::Path;

use super::message::{Message, ReadMessage, Version};

pub struct Parser {
    version: Version,
    cursor: Cursor<Vec<u8>>,
}

impl Read for Parser {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.cursor.read(buf)
    }
}

impl ReadMessage for Parser {
    fn version(&self) -> &Version {
        &self.version
    }
}

impl Parser {
    pub fn new<P: AsRef<Path>>(filepath: P, version: Version) -> Self {
        // NOTE: The current approach loads the entire file content into memory
        // TODO: Read and process the file content in smaller chunks
        let buffer = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(buffer);

        Self { version, cursor }
    }

    pub fn get_next_message(&mut self) -> Result<Message> {
        loop {
            // TODO: Add logic to handle reaching the end of the buffer
            let size = self.read_size()?;
            let kind = self.read_kind()?;

            let msg = match kind {
                'T' => self.parse_timestamp()?,
                'S' => self.parse_system_event()?,
                'A' => self.parse_add_order()?,
                'E' => self.parse_execute_order()?,
                'X' => self.parse_cancel_order()?,
                'D' => self.parse_delete_order()?,
                'U' => self.parse_replace_order()?,
                _ => None,
            };

            match msg {
                Some(m) => return Ok(m),
                None => {
                    self.skip_message(size)?;
                    continue;
                }
            }
        }
    }

    fn parse_timestamp(&mut self) -> Result<Option<Message>> {
        let seconds = self.read_seconds()?;

        let message = Message::Timestamp { seconds };

        Ok(Some(message))
    }

    fn parse_system_event(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.read_nanoseconds()?;
        let event_code = self.read_event_code()?;

        let message = Message::SystemEvent {
            nanoseconds,
            event_code,
        };

        Ok(Some(message))
    }

    fn parse_add_order(&mut self) -> Result<Option<Message>> {
        // TODO: Return `None` if the ticker is not a target

        let nanoseconds = self.read_nanoseconds()?;
        let refno = self.read_refno()?;
        let side = self.read_side()?;
        let shares = self.read_shares()?;
        let ticker = self.read_ticker()?;
        let price = self.read_price()?;

        let message = Message::AddOrder {
            nanoseconds,
            refno,
            side,
            shares,
            ticker,
            price,
        };

        Ok(Some(message))
    }

    fn parse_execute_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.read_nanoseconds()?;
        let refno = self.read_refno()?;
        let shares = self.read_shares()?;
        let matchno = self.read_matchno()?;

        let message = Message::ExecuteOrder {
            nanoseconds,
            refno,
            shares,
            matchno,
        };

        Ok(Some(message))
    }

    fn parse_cancel_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.read_nanoseconds()?;
        let refno = self.read_refno()?;
        let shares = self.read_shares()?;

        let message = Message::CancelOrder {
            nanoseconds,
            refno,
            shares,
        };

        Ok(Some(message))
    }

    fn parse_delete_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.read_nanoseconds()?;
        let refno = self.read_refno()?;

        let message = Message::DeleteOrder { nanoseconds, refno };

        Ok(Some(message))
    }

    fn parse_replace_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.read_nanoseconds()?;
        let refno = self.read_refno()?;
        let new_refno = self.read_new_refno()?;
        let shares = self.read_shares()?;
        let price = self.read_price()?;

        let message = Message::ReplaceOrder {
            nanoseconds,
            refno,
            new_refno,
            shares,
            price,
        };

        Ok(Some(message))
    }

    fn skip_message(&mut self, size: u16) -> Result<()> {
        // NOTE: Assumes the message has been read up to the type byte
        let offset = (size - 1) as i64;
        self.cursor.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}
