use std::{
    collections::{HashSet, VecDeque},
    io::{Read, Result, Seek, SeekFrom},
};

use crate::{
    buffer::Peek,
    constants::EVERY_TICKER,
    message::{
        peek_kind, peek_refno, peek_ticker, read_kind, read_replace_order, read_seconds, read_size,
        AddOrder, BrokenTrade, CancelOrder, Context, CrossTrade, DeleteOrder, ExecuteOrder,
        Message, NetOrderImbalanceIndicator, ReadMessage, SystemEvent, Trade, Version,
    },
};

pub struct Reader {
    version: Version,
    tickers: HashSet<String>,
    context: Context,
    buf: VecDeque<Message>, // To handle the case where multiple messages are parsed at once
}

impl Reader {
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
                // system event message
                'S' => self.parse_system_event(buffer)?,
                // order messages
                'A' | 'F' => self.parse_add_order(buffer)?,
                'E' | 'C' => self.parse_execute_order(buffer)?,
                'X' => self.parse_cancel_order(buffer)?,
                'D' => self.parse_delete_order(buffer)?,
                'U' => self.parse_replace_order(buffer)?,
                // trade messages
                'P' => self.parse_trade(buffer)?,
                'Q' => self.parse_cross_trade(buffer)?,
                'B' => self.parse_broken_trade(buffer)?,
                // net order imbalance message
                'I' => self.parse_noii(buffer)?,
                _ => None,
            };

            match msg {
                Some(m) => return Ok(m),
                None => {
                    buffer.seek(SeekFrom::Current(size as i64))?;
                    buffer.peek(0, 1)?;
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
            let ticker = peek_ticker(buffer, 18, &self.version)?;
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

    fn parse_trade<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let ticker = peek_ticker(buffer, 18, &self.version)?;

        if self.tickers.contains(&ticker) || self.tickers.contains(EVERY_TICKER) {
            let message = Trade::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::Trade(message)))
        } else {
            Ok(None)
        }
    }

    fn parse_cross_trade<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let ticker = peek_ticker(buffer, 13, &self.version)?;

        if self.tickers.contains(&ticker) || self.tickers.contains(EVERY_TICKER) {
            let message = CrossTrade::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::CrossTrade(message)))
        } else {
            Ok(None)
        }
    }

    fn parse_broken_trade<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        // Broken trades don't have tickers, so we always parse them
        let message = BrokenTrade::read(buffer, &self.version, &mut self.context)?;
        Ok(Some(Message::BrokenTrade(message)))
    }

    fn parse_noii<T>(&mut self, buffer: &mut T) -> Result<Option<Message>>
    where
        T: Read + Seek + Peek,
    {
        let ticker = peek_ticker(buffer, 22, &self.version)?;

        if self.tickers.contains(&ticker) || self.tickers.contains(EVERY_TICKER) {
            let message =
                NetOrderImbalanceIndicator::read(buffer, &self.version, &mut self.context)?;
            Ok(Some(Message::NetOrderImbalanceIndicator(message)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::{prelude::FileWriteBin, NamedTempFile};

    use super::*;
    use crate::{
        buffer::BufFile,
        message::{test_helpers::message_builders::*, Side},
    };

    #[test]
    // extract_message returns an err if there are no more relevant messages
    fn returns_an_error() {
        // create a temporary file
        let sinkfile = NamedTempFile::new("test_messages.bin").unwrap();

        // create a reader with *no* tickers
        let mut reader = Reader::new(Version::V41, HashSet::new());
        reader.context.update_clock(0);

        // add messages to the file
        // let messages = vec![];
        let messages = vec![
            add_order_v41(0, 89402372340, Side::Buy, 100, "X", 1000),
            add_order_v41(0, 23451234098, Side::Buy, 100, "Y", 1000),
            add_order_v41(0, 98290347401, Side::Buy, 100, "Z", 1000),
        ];
        let data = create_message_sequence(messages);
        sinkfile.write_binary(&data.into_inner()).unwrap();
        let mut buffile = BufFile::new(sinkfile.path()).unwrap();

        // extract the next message
        // let message = reader.extract_message_exactly(&mut buffile);
        let message = reader.extract_message(&mut buffile);

        // check that the result is an error
        assert!(message.is_err());
    }

    #[test]
    // extract_message updates clock if the next message is type 'T'
    fn updates_clock() {
        // create a temporary file
        let sinkfile = NamedTempFile::new("test_messages.bin").unwrap();

        // create a reader with a ticker
        let tickers = HashSet::from(["A".to_string()]);
        let mut reader = Reader::new(Version::V41, tickers);
        reader.context.update_clock(0);

        // add messages to the file
        let messages = vec![
            timestamp_v41(3600),
            system_event_v41(0, 'O'),
            add_order_v41(0, 89402372340, Side::Buy, 100, "A", 1000),
        ];
        let data = create_message_sequence(messages);
        sinkfile.write_binary(&data.into_inner()).unwrap();
        let mut buffile = BufFile::new(sinkfile.path()).unwrap();

        // extract the next message
        // let message = reader.extract_message_exactly(&mut buffile).unwrap();
        let message = reader.extract_message(&mut buffile).unwrap();

        // check that the clock updated
        assert_eq!(reader.context.clock.unwrap(), 3600);

        // check that the return message matches the system message
        // assert!(message.is_none());
        assert!(matches!(message, Message::SystemEvent(_)));
    }

    #[test]
    // extract_message ignores add orders for refnos not in self.tickers
    fn ignores_add_order_tickers() {
        // create a temporary file
        let sinkfile = NamedTempFile::new("test_messages.bin").unwrap();

        // create a reader with a ticker
        let tickers = HashSet::from(["A".to_string()]);
        let mut reader = Reader::new(Version::V41, tickers);
        reader.context.update_clock(0);

        // add messages to the file
        let messages = vec![
            add_order_v41(0, 89402372340, Side::Buy, 100, "X", 1000),
            add_order_v41(0, 9234509829, Side::Buy, 100, "A", 1000),
        ];
        let data = create_message_sequence(messages);
        sinkfile.write_binary(&data.into_inner()).unwrap();
        let mut buffile = BufFile::new(sinkfile.path()).unwrap();

        // extract the next message
        // let message = reader.extract_message_exactly(&mut buffile).unwrap();
        let message = reader.extract_message(&mut buffile).unwrap();

        // check that the return message matches the system message
        // assert!(message.is_none());
        if let Message::AddOrder(add_order) = message {
            assert_eq!(add_order.ticker, "A");
        }
    }

    #[test]
    // extract_message ignores modify orders for refnos not in self.context
    fn ignores_modify_order_refnos() {
        // create a temporary file
        let sinkfile = NamedTempFile::new("test_messages.bin").unwrap();

        // create a reader with a ticker
        let tickers = HashSet::from(["A".to_string()]);
        let mut reader = Reader::new(Version::V41, tickers);
        reader.context.update_clock(0);
        let order = create_order_state("A", Side::Buy, 1000, 200);
        reader.context.active_orders.insert(89402372340, order);

        // add messages to the file
        let messages = vec![
            cancel_order_v41(0, 97890234892, 100),
            cancel_order_v41(0, 89402372340, 100),
        ];
        let data = create_message_sequence(messages);
        sinkfile.write_binary(&data.into_inner()).unwrap();
        let mut buffile = BufFile::new(sinkfile.path()).unwrap();

        // extract the next message
        // let message = reader.extract_message_exactly(&mut buffile).unwrap();
        let message = reader.extract_message(&mut buffile).unwrap();

        // check that the return message matches the system message
        // assert!(message.is_none());
        if let Message::CancelOrder(cancel_order) = message {
            assert_eq!(cancel_order.refno, 89402372340);
        }
    }
}
