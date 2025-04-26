use std::io::{Result, Seek, SeekFrom};

use crate::buffer::Buffer;

use super::{read_event_code, read_nanoseconds};
use super::{Context, EventCode, ReadMessage, Version};

#[derive(Debug)]
pub struct SystemEvent {
    nanoseconds: u64,
    event_code: EventCode,
}

impl ReadMessage for SystemEvent {
    fn read<const N: usize>(
        buffer: &mut Buffer<N>,
        version: &Version,
        context: &Context,
    ) -> Result<Self> {
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }

        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let event_code = read_event_code(buffer)?;

        Ok(Self {
            nanoseconds,
            event_code,
        })
    }
}
