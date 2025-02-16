use byteorder::{NetworkEndian, ReadBytesExt};
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    TimeStamp,
    SystemEvent,
    AddOrder,
    ExecuteOrder,
    CancelOrder,
    DeleteOrder,
    ReplaceOrder,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventCode {
    StartMessages,
    StartSystem,
    StartMarketHours,
    EndMarketHours,
    EndSystem,
    EndMessages,
    EmergencyMarketHalt,
    EmergencyMarketQuoteOnly,
    EmergencyMarketResumption,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Message {
    kind: Option<MessageType>,
    seconds: Option<u32>,
    nanoseconds: Option<u32>,
    event_code: Option<EventCode>,
    refno: Option<u64>,
    side: Option<Side>,
    shares: Option<u32>,
    ticker: Option<String>,
    price: Option<u32>,
    matchno: Option<u64>,
    new_refno: Option<u64>,
}

impl Message {
    fn new() -> Self {
        Self {
            kind: None,
            seconds: None,
            nanoseconds: None,
            event_code: None,
            refno: None,
            side: None,
            shares: None,
            ticker: None,
            price: None,
            matchno: None,
            new_refno: None,
        }
    }

    fn reset(&mut self) {
        self.kind = None;
        self.seconds = None;
        self.nanoseconds = None;
        self.event_code = None;
        self.refno = None;
        self.side = None;
        self.shares = None;
        self.ticker = None;
        self.price = None;
        self.matchno = None;
        self.new_refno = None;
    }

    fn set_kind(&mut self, kind: MessageType) {
        self.kind = Some(kind);
    }

    fn set_seconds(&mut self, seconds: u32) {
        self.seconds = Some(seconds);
    }

    fn set_nanoseconds(&mut self, nanoseconds: u32) {
        self.nanoseconds = Some(nanoseconds);
    }

    fn set_event_code(&mut self, event_code: EventCode) {
        self.event_code = Some(event_code);
    }

    fn set_refno(&mut self, refno: u64) {
        self.refno = Some(refno);
    }

    fn set_side(&mut self, side: Side) {
        self.side = Some(side);
    }

    fn set_shares(&mut self, shares: u32) {
        self.shares = Some(shares);
    }

    fn set_ticker(&mut self, ticker: String) {
        self.ticker = Some(ticker);
    }

    fn set_price(&mut self, price: u32) {
        self.price = Some(price);
    }

    fn set_matchno(&mut self, matchno: u64) {
        self.matchno = Some(matchno);
    }

    fn set_new_refno(&mut self, new_refno: u64) {
        self.new_refno = Some(new_refno);
    }

    pub fn kind(&self) -> Option<MessageType> {
        self.kind
    }

    pub fn seconds(&self) -> Option<u32> {
        self.seconds
    }

    pub fn nanoseconds(&self) -> Option<u32> {
        self.nanoseconds
    }

    pub fn event_code(&self) -> Option<EventCode> {
        self.event_code
    }

    pub fn refno(&self) -> Option<u64> {
        self.refno
    }

    pub fn side(&self) -> Option<Side> {
        self.side
    }

    pub fn shares(&self) -> Option<u32> {
        self.shares
    }

    pub fn ticker(&self) -> Option<String> {
        self.ticker.clone()
    }

    pub fn price(&self) -> Option<u32> {
        self.price
    }

    pub fn matchno(&self) -> Option<u64> {
        self.matchno
    }

    pub fn new_refno(&self) -> Option<u64> {
        self.new_refno
    }
}

trait ReadString: Read {
    fn read_utf8_string(&mut self, string_length: usize) -> Result<String> {
        let mut buf = vec![0; string_length];
        self.read_exact(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }
}

impl ReadString for Cursor<Vec<u8>> {}

pub struct Parser {
    cursor: Cursor<Vec<u8>>,
    pub current_message: Message,
}

impl Parser {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Self {
        // NOTE: The current approach loads the entire file content into memory
        // TODO: Explore alternatives to read and process the file content in smaller chunks
        let buffer = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(buffer);

        let current_message = Message::new();

        Self {
            cursor,
            current_message,
        }
    }

    pub fn next(&mut self) -> Result<()> {
        loop {
            // TODO: Add logic to handle reaching the end of the buffer
            let size = self.cursor.read_u16::<NetworkEndian>()?;
            match char::from(self.cursor.read_u8()?) {
                'T' => {
                    self.read_message(MessageType::TimeStamp)?;
                    break;
                }
                'S' => {
                    self.read_message(MessageType::SystemEvent)?;
                    break;
                }
                'A' => {
                    // TODO: Implement ticker-based skip logic
                    self.read_message(MessageType::AddOrder)?;
                    break;
                }
                'E' => {
                    self.read_message(MessageType::ExecuteOrder)?;
                    break;
                }
                'X' => {
                    self.read_message(MessageType::CancelOrder)?;
                    break;
                }
                'D' => {
                    self.read_message(MessageType::DeleteOrder)?;
                    break;
                }
                'U' => {
                    self.read_message(MessageType::ReplaceOrder)?;
                    break;
                }
                _ => {
                    self.skip_message(size)?;
                    continue;
                }
            }
        }
        Ok(())
    }

    fn read_message(&mut self, kind: MessageType) -> Result<()> {
        self.current_message.reset();
        self.current_message.set_kind(kind);

        // NOTE: Assumes the message has been read up to the type byte
        match kind {
            MessageType::TimeStamp => {
                let seconds = self.cursor.read_u32::<NetworkEndian>()?;
                self.current_message.set_seconds(seconds);
            }
            MessageType::SystemEvent => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>()?;
                let event_code = match char::from(self.cursor.read_u8()?) {
                    'O' => EventCode::StartMessages,
                    'S' => EventCode::StartSystem,
                    'Q' => EventCode::StartMarketHours,
                    'M' => EventCode::EndMarketHours,
                    'E' => EventCode::EndSystem,
                    'C' => EventCode::EndMessages,
                    'A' => EventCode::EmergencyMarketHalt,
                    'R' => EventCode::EmergencyMarketQuoteOnly,
                    'B' => EventCode::EmergencyMarketResumption,
                    unknown_code => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid event code encountered: {}", unknown_code),
                        ));
                    }
                };
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_event_code(event_code);
            }
            MessageType::AddOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>()?;
                let refno = self.cursor.read_u64::<NetworkEndian>()?;
                let side = match char::from(self.cursor.read_u8()?) {
                    'B' => Side::Buy,
                    'S' => Side::Sell,
                    unknown_code => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid code for trading: {}", unknown_code),
                        ));
                    }
                };
                let shares = self.cursor.read_u32::<NetworkEndian>()?;
                let ticker = self.cursor.read_utf8_string(8)?;
                let price = self.cursor.read_u32::<NetworkEndian>()?;
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_side(side);
                self.current_message.set_shares(shares);
                self.current_message.set_ticker(ticker);
                self.current_message.set_price(price);
            }
            MessageType::ExecuteOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>()?;
                let refno = self.cursor.read_u64::<NetworkEndian>()?;
                let shares = self.cursor.read_u32::<NetworkEndian>()?;
                let matchno = self.cursor.read_u64::<NetworkEndian>()?;
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_shares(shares);
                self.current_message.set_matchno(matchno);
            }
            MessageType::CancelOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>()?;
                let refno = self.cursor.read_u64::<NetworkEndian>()?;
                let shares = self.cursor.read_u32::<NetworkEndian>()?;
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_shares(shares);
            }
            MessageType::DeleteOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>()?;
                let refno = self.cursor.read_u64::<NetworkEndian>()?;
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
            }
            MessageType::ReplaceOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>()?;
                let refno = self.cursor.read_u64::<NetworkEndian>()?;
                let new_refno = self.cursor.read_u64::<NetworkEndian>()?;
                let shares = self.cursor.read_u32::<NetworkEndian>()?;
                let price = self.cursor.read_u32::<NetworkEndian>()?;
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_new_refno(new_refno);
                self.current_message.set_shares(shares);
                self.current_message.set_price(price);
            }
        }
        Ok(())
    }

    fn skip_message(&mut self, size: u16) -> Result<()> {
        // NOTE: Assumes the message has been read up to the type byte
        let offset = (size - 1) as i64;
        self.cursor.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}
