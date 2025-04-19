use std::io::Result;

use byteorder::{NetworkEndian, ReadBytesExt};

use super::buffer::Buffer;
use super::message::{
    AddOrder, CancelOrder, DeleteOrder, ExecuteOrder, ReplaceOrder, SystemEvent, Timestamp,
};
use super::message::{Message, ReadMessage, Version};

pub struct Parser {
    version: Version,
}

impl Parser {
    pub fn new(version: Version) -> Self {
        Self { version }
    }

    pub fn extract_message(&self, buffer: &mut Buffer) -> Result<Message> {
        loop {
            // TODO: Add logic to handle reaching the end of the buffer
            let size = buffer.read_u16::<NetworkEndian>()?;
            let kind = buffer.read_u8().map(char::from)?;

            if let Version::V50 = self.version {
                buffer.skip(32)?; // Discard stock locate and tracking number
            }

            let msg = match kind {
                'T' => {
                    let data = Timestamp::read(buffer, &self.version)?;
                    Some(Message::Timestamp(data))
                }
                'S' => {
                    let data = SystemEvent::read(buffer, &self.version)?;
                    Some(Message::SystemEvent(data))
                }
                'A' => {
                    // TODO: Return `None` if the ticker is not a target
                    let data = AddOrder::read(buffer, &self.version)?;
                    Some(Message::AddOrder(data))
                }
                'E' => {
                    let data = ExecuteOrder::read(buffer, &self.version)?;
                    Some(Message::ExecuteOrder(data))
                }
                'X' => {
                    let data = CancelOrder::read(buffer, &self.version)?;
                    Some(Message::CancelOrder(data))
                }
                'D' => {
                    let data = DeleteOrder::read(buffer, &self.version)?;
                    Some(Message::DeleteOrder(data))
                }
                'U' => {
                    let data = ReplaceOrder::read(buffer, &self.version)?;
                    Some(Message::ReplaceOrder(data))
                }
                _ => None,
            };

            match msg {
                Some(m) => return Ok(m),
                None => {
                    buffer.skip(size)?;
                    continue;
                }
            }
        }
    }
}
