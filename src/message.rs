use std::io::{Error, ErrorKind, Read, Result};

use byteorder::{NetworkEndian, ReadBytesExt};

#[derive(Debug, PartialEq)]
pub enum Version {
    V41, // Version 4.1
    V50, // Version 5.0
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum Message {
    Timestamp {
        seconds: u32,
    },
    SystemEvent {
        nanoseconds: u32,
        event_code: EventCode,
    },
    AddOrder {
        nanoseconds: u32,
        refno: u64,
        side: Side,
        shares: u32,
        ticker: String,
        price: u32,
    },
    ExecuteOrder {
        nanoseconds: u32,
        refno: u64,
        shares: u32,
        matchno: u64,
    },
    CancelOrder {
        nanoseconds: u32,
        refno: u64,
        shares: u32,
    },
    DeleteOrder {
        nanoseconds: u32,
        refno: u64,
    },
    ReplaceOrder {
        nanoseconds: u32,
        refno: u64,
        new_refno: u64,
        shares: u32,
        price: u32,
    },
}

pub trait ReadMessage: Read {
    fn version(&self) -> &Version;

    fn read_size(&mut self) -> Result<u16> {
        self.read_u16::<NetworkEndian>()
    }

    fn read_kind(&mut self) -> Result<char> {
        self.read_u8().map(char::from)
    }

    fn read_seconds(&mut self) -> Result<u32> {
        self.read_u32::<NetworkEndian>()
    }

    fn read_nanoseconds(&mut self) -> Result<u32> {
        self.read_u32::<NetworkEndian>()
    }

    fn read_shares(&mut self) -> Result<u32> {
        self.read_u32::<NetworkEndian>()
    }

    fn read_price(&mut self) -> Result<u32> {
        self.read_u32::<NetworkEndian>()
    }

    fn read_refno(&mut self) -> Result<u64> {
        self.read_u64::<NetworkEndian>()
    }

    fn read_new_refno(&mut self) -> Result<u64> {
        self.read_u64::<NetworkEndian>()
    }

    fn read_matchno(&mut self) -> Result<u64> {
        self.read_u64::<NetworkEndian>()
    }

    fn read_side(&mut self) -> Result<Side> {
        let side = match self.read_u8().map(char::from)? {
            'B' => Side::Buy,
            'S' => Side::Sell,
            unknown_code => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid code for trading: {}", unknown_code),
                ));
            }
        };
        Ok(side)
    }

    fn read_event_code(&mut self) -> Result<EventCode> {
        let event_code = match self.read_u8().map(char::from)? {
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
        Ok(event_code)
    }

    fn read_ticker(&mut self) -> Result<String> {
        let mut buf = vec![0; 8];
        self.read_exact(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }
}
