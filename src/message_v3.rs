use byteorder::{NetworkEndian, ReadBytesExt};
use std::io::{Cursor, Result};
use std::path::Path;

pub enum MessageType {
    TimeStamp,
    SystemEvent,
    AddOrder,
    ExecuteOrder,
    CancelOrder,
    DeleteOrder,
    ReplaceOrder,
    Unknown,
}

pub struct Message {
    cursor: Cursor<Vec<u8>>,
    pos: u64, // Starting position of the current message
}

impl Message {
    pub fn new(filepath: &Path) -> Self {
        let buffer = std::fs::read(filepath).expect("Unable to read file");
        let cursor = Cursor::new(buffer);
        let pos = 0;
        Self { cursor, pos }
    }

    fn size(&mut self) -> u16 {
        self.cursor.set_position(self.pos);
        self.cursor.read_u16::<NetworkEndian>().unwrap()
    }

    pub fn kind(&mut self) -> MessageType {
        self.cursor.set_position(self.pos + 2);
        match char::from(self.cursor.read_u8().unwrap()) {
            'T' => MessageType::TimeStamp,
            'S' => MessageType::SystemEvent,
            'A' => MessageType::AddOrder,
            'E' => MessageType::ExecuteOrder,
            'X' => MessageType::CancelOrder,
            'D' => MessageType::DeleteOrder,
            'U' => MessageType::ReplaceOrder,
            _ => MessageType::Unknown,
        }
    }

    pub fn seconds(&mut self) -> Result<Option<u32>> {
        let offset: u64 = match self.kind() {
            MessageType::TimeStamp => 2 + 1,
            _ => 0,
        };

        if offset == 0 {
            return Ok(None);
        }

        self.cursor.set_position(self.pos + offset);
        self.cursor.read_u32::<NetworkEndian>().map(Some)
    }

    pub fn next(&mut self) {
        let offset = self.size() as u64;
        self.cursor.set_position(self.pos + offset);
        self.pos = self.cursor.position();
    }

    pub fn serialize() {
        todo!("Return a materialized serialization of the current message");
    }
}
