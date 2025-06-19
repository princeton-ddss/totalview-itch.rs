use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_event_code, read_kind, read_nanoseconds};
use super::{Context, EventCode, ReadMessage, Version};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SystemEvent {
    nanoseconds: u64,
    kind: char,
    event_code: EventCode,
}

impl ReadMessage for SystemEvent {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        // Read data from buffer
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let event_code = read_event_code(buffer)?;

        // Return message
        Ok(Self {
            nanoseconds,
            kind,
            event_code,
        })
    }
}
