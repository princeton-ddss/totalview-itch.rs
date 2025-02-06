use std::io::Cursor;
use std::path::Path;

#[derive(Clone)]
pub enum MessageType {
    Time,
    System,
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

    fn set_time_message(&mut self, seconds: u32) {
        self.reset();
        self.kind = Some(MessageType::Time);
        self.seconds = Some(seconds);
    }

    pub fn kind(&self) -> Option<MessageType> {
        self.kind.clone()
    }

    pub fn seconds(&self) -> Option<u32> {
        self.seconds
    }

    pub fn ticker(&self) -> Option<String> {
        self.ticker.clone()
    }
}

pub struct Parser {
    cursor: Cursor<Vec<u8>>,
    pub current_message: Message,
}

impl Parser {
    pub fn new(filepath: &Path) -> Self {
        let buffer = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(buffer);

        let current_message = Message::new();

        Self {
            cursor,
            current_message,
        }
    }

    pub fn next(&mut self) {}
}
