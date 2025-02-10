use byteorder::{NetworkEndian, ReadBytesExt};
use std::io::{Cursor, Seek, SeekFrom};
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
}

pub struct Parser {
    cursor: Cursor<Vec<u8>>,
    pub current_message: Message,
}

impl Parser {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Self {
        let buffer = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(buffer);

        let current_message = Message::new();

        Self {
            cursor,
            current_message,
        }
    }

    pub fn next(&mut self) {
        loop {
            let size = self.cursor.read_u16::<NetworkEndian>().unwrap();
            match char::from(self.cursor.read_u8().unwrap()) {
                'T' => {
                    self.read_message(MessageType::TimeStamp);
                    break;
                }
                'S' => {
                    self.read_message(MessageType::SystemEvent);
                    break;
                }
                'A' => {
                    // TODO: Implement ticker-based skip logic
                    self.read_message(MessageType::AddOrder);
                    break;
                }
                'E' => {
                    // TODO: Implement ticker-based skip logic
                    self.read_message(MessageType::ExecuteOrder);
                    break;
                }
                'X' => {
                    // TODO: Implement ticker-based skip logic
                    self.read_message(MessageType::CancelOrder);
                    break;
                }
                'D' => {
                    // TODO: Implement ticker-based skip logic
                    self.read_message(MessageType::DeleteOrder);
                    break;
                }
                'U' => {
                    // TODO: Implement ticker-based skip logic
                    self.read_message(MessageType::ReplaceOrder);
                    break;
                }
                _ => {
                    self.skip_message(size);
                    continue;
                }
            }
        }
    }

    fn read_message(&mut self, kind: MessageType) {
        self.current_message.reset();
        self.current_message.set_kind(kind);

        // NOTE: Assumes the message has been read up to the type byte
        match kind {
            MessageType::TimeStamp => {
                let seconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_seconds(seconds);
            }
            MessageType::SystemEvent => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                let event_code = match char::from(self.cursor.read_u8().unwrap()) {
                    'O' => EventCode::StartMessages,
                    'S' => EventCode::StartSystem,
                    'Q' => EventCode::StartMarketHours,
                    'M' => EventCode::EndMarketHours,
                    'E' => EventCode::EndSystem,
                    'C' => EventCode::EndMessages,
                    'A' => EventCode::EmergencyMarketHalt,
                    'R' => EventCode::EmergencyMarketQuoteOnly,
                    'B' => EventCode::EmergencyMarketResumption,
                    e => panic!("Invalid event code encountered: {}", e),
                };
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_event_code(event_code);
            }
            MessageType::AddOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
                let side = match char::from(self.cursor.read_u8().unwrap()) {
                    'B' => Side::Buy,
                    'S' => Side::Sell,
                    e => panic!("Invalid code for trading: {}", e),
                };
                let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_side(side);
                self.current_message.set_shares(shares);
            }
            MessageType::ExecuteOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
                let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_shares(shares);
            }
            MessageType::CancelOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
                let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_shares(shares);
            }
            MessageType::DeleteOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
            }
            MessageType::ReplaceOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
                let new_refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
                let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
                self.current_message.set_refno(refno);
                self.current_message.set_shares(shares);
            }
        }
    }

    fn skip_message(&mut self, size: u16) {
        // NOTE: Assumes the message has been read up to the type byte
        let offset = (size - 1) as i64;
        self.cursor.seek(SeekFrom::Current(offset)).unwrap();
    }
}
