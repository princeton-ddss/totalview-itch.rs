mod message;

use std::io::{Cursor, Result, Seek, SeekFrom};
use std::path::Path;

use message::{Message, ReadMessage};

impl ReadMessage for Cursor<Vec<u8>> {}

pub struct Parser {
    cursor: Cursor<Vec<u8>>,
    messages: Vec<Message>,
}

impl Parser {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Self {
        // NOTE: The current approach loads the entire file content into memory
        // TODO: Read and process the file content in smaller chunks
        let buffer = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(buffer);

        let messages = vec![];

        Self { cursor, messages }
    }

    pub fn get_current_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn next(&mut self) -> Result<()> {
        loop {
            // TODO: Add logic to handle reaching the end of the buffer
            let size = self.cursor.read_size()?;
            let kind = self.cursor.read_kind()?;

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
                Some(m) => {
                    self.messages.push(m);
                    break;
                }
                None => {
                    self.skip_message(size)?;
                    continue;
                }
            }
        }
        Ok(())
    }

    fn parse_timestamp(&mut self) -> Result<Option<Message>> {
        let seconds = self.cursor.read_seconds()?;

        let message = Message::Timestamp { seconds };

        Ok(Some(message))
    }

    fn parse_system_event(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.cursor.read_nanoseconds()?;
        let event_code = self.cursor.read_event_code()?;

        let message = Message::SystemEvent {
            nanoseconds,
            event_code,
        };

        Ok(Some(message))
    }

    fn parse_add_order(&mut self) -> Result<Option<Message>> {
        // TODO: Return `None` if the ticker is not a target

        let nanoseconds = self.cursor.read_nanoseconds()?;
        let refno = self.cursor.read_refno()?;
        let side = self.cursor.read_side()?;
        let shares = self.cursor.read_shares()?;
        let ticker = self.cursor.read_ticker()?;
        let price = self.cursor.read_price()?;

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
        let nanoseconds = self.cursor.read_nanoseconds()?;
        let refno = self.cursor.read_refno()?;
        let shares = self.cursor.read_shares()?;
        let matchno = self.cursor.read_matchno()?;

        let message = Message::ExecuteOrder {
            nanoseconds,
            refno,
            shares,
            matchno,
        };

        Ok(Some(message))
    }

    fn parse_cancel_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.cursor.read_nanoseconds()?;
        let refno = self.cursor.read_refno()?;
        let shares = self.cursor.read_shares()?;

        let message = Message::CancelOrder {
            nanoseconds,
            refno,
            shares,
        };

        Ok(Some(message))
    }

    fn parse_delete_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.cursor.read_nanoseconds()?;
        let refno = self.cursor.read_refno()?;

        let message = Message::DeleteOrder { nanoseconds, refno };

        Ok(Some(message))
    }

    fn parse_replace_order(&mut self) -> Result<Option<Message>> {
        let nanoseconds = self.cursor.read_nanoseconds()?;
        let refno = self.cursor.read_refno()?;
        let new_refno = self.cursor.read_new_refno()?;
        let shares = self.cursor.read_shares()?;
        let price = self.cursor.read_price()?;

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
