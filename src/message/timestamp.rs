use std::io::{Error, ErrorKind, Result};

use crate::buffer::Buffer;

use super::read_seconds;
use super::{ReadMessage, Version};

#[derive(Debug)]
pub struct Timestamp {
    seconds: u32,
}

impl ReadMessage for Timestamp {
    fn read<const N: usize>(buffer: &mut Buffer<N>, version: &Version) -> Result<Self> {
        if version != &Version::V41 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{version} does not support <Timestamp> message"),
            ));
        }

        let seconds = read_seconds(buffer, version)?;

        Ok(Self { seconds })
    }
}
