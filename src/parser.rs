use std::collections::{HashSet, VecDeque};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

use crate::buffer::Peek;
use crate::constants::EVERY_TICKER;
use crate::message::{peek_kind, peek_refno, peek_ticker, read_kind, read_seconds, read_size};
use crate::message::{
    read_replace_order, AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, SystemEvent,
};
use crate::message::{Context, Message, ReadMessage, Version};

pub struct Parser {
    version: Version,
    tickers: HashSet<String>,
    context: Context,
    buf: VecDeque<Message>, // To handle the case where multiple messages are parsed at once
}

impl Parser {
    pub fn new(version: Version, tickers: HashSet<String>) -> Self {
        Self {
            version,
            tickers,
            context: Context::new(),
            buf: VecDeque::new(),
        }
    }

    pub fn extract_message<T>(&mut self, buffer: &mut T) -> Result<Message>
    where
        T: Read + Seek + Peek,
    {
        if !self.buf.is_empty() {
            return Ok(self.buf.pop_front().unwrap());
        }

        loop {
            let size = read_size(buffer)?;
            let kind = peek_kind(buffer)?;

            if kind == 'T' {
                let _kind = read_kind(buffer)?;
                let seconds = read_seconds(buffer)?;
                self.context.update_clock(seconds);
                continue;
            }

            let msg = match kind {
                'S' => self.parse_system_event(buffer)?,
                'A' | 'F' => self.parse_add_order(buffer)?,
                'E' | 'C' => self.parse_execute_order(buffer)?,
                'X' => self.parse_cancel_order(buffer)?,
                'D' => self.parse_delete_order(buffer)?,
                'U' => self.parse_replace_order(buffer)?,
                _ => None,
            };

            match msg {
                Some(m) => return Ok(m),
                None => {
                    buffer.seek(SeekFrom::Current(size as i64))?;
                    match buffer.peek_ahead(0, 1) {
                        Err(_) => {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "File stream is complete.",
                            ))
                        }
                        Ok(_) => continue,
                    }
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
        let should_parse = if self.tickers.contains(EVERY_TICKER) {
            true
        } else {
            let ticker = peek_ticker(buffer, &self.version)?;
            self.tickers.contains(&ticker)
        };

        if should_parse {
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
        let refno = peek_refno(buffer, &self.version)?;
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
        let refno = peek_refno(buffer, &self.version)?;
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
        let refno = peek_refno(buffer, &self.version)?;
        if self.context.has_order(refno) {
            let data = DeleteOrder::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::DeleteOrder(data)))
        } else {
            Ok(None)
        }
    }

    // Why not return a ReplaceOrde and deal with splitting it outside? The deque works, but it is
    // a bit strange...
    fn parse_replace_order<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let refno = peek_refno(buffer, &self.version)?;
        if self.context.has_order(refno) {
            let (delete_order, add_order) =
                read_replace_order(buffer, &self.version, &mut self.context)?;
            self.buf.push_back(Message::AddOrder(add_order)); // Return in next call
            Ok(Some(Message::DeleteOrder(delete_order)))
        } else {
            Ok(None)
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn returns_an_error() {
        // extract_message returns an err when there are no more supported messages
    }

    #[test]
    fn updates_clock() {
        // extract_message updates clock if the next message is type 'T'
    }

    #[test]
    fn ignores_tickers() {
        // extract_messages ignores add orders for tickers not in self.tickers
    }

    #[test]
    fn ignores_refnos() {
        // extract_messages ignores modify orders for refnos not in self.context
    }
}
