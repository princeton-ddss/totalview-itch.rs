use byteorder::{NetworkEndian, ReadBytesExt};
use std::io::{Cursor, Seek, SeekFrom};
use std::path::Path;

#[derive(Clone, Copy)]
pub enum MessageType {
    TimeStamp,
    SystemEvent,
    AddOrder,
    ExecuteOrder,
    CancelOrder,
    DeleteOrder,
    ReplaceOrder,
}

pub struct Message {
    kind: Option<MessageType>,
    seconds: Option<u32>,
    nanoseconds: Option<u32>,
    event_code: Option<u8>,
    refno: Option<u64>,
    side: Option<u8>,
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

    pub fn kind(&self) -> Option<MessageType> {
        self.kind
    }

    pub fn seconds(&self) -> Option<u32> {
        self.seconds
    }

    pub fn nanoseconds(&self) -> Option<u32> {
        self.nanoseconds
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
                    self.read_message(MessageType::AddOrder);
                    break;
                }
                'E' => {
                    self.read_message(MessageType::ExecuteOrder);
                    break;
                }
                'X' => {
                    self.read_message(MessageType::CancelOrder);
                    break;
                }
                'D' => {
                    self.read_message(MessageType::DeleteOrder);
                    break;
                }
                'U' => {
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
                self.current_message.set_nanoseconds(nanoseconds);
            }
            MessageType::AddOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
            }
            MessageType::ExecuteOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
            }
            MessageType::CancelOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
            }
            MessageType::DeleteOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
            }
            MessageType::ReplaceOrder => {
                let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
                self.current_message.set_nanoseconds(nanoseconds);
            }
        }
    }

    fn skip_message(&mut self, size: u16) {
        // NOTE: Assumes the message has been read up to the type byte
        let offset = (size - 1) as i64;
        self.cursor.seek(SeekFrom::Current(offset)).unwrap();
    }
}
