use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{
    read_event_code, read_kind, read_nanoseconds, Context, EventCode, ReadMessage, Version,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{test_helpers::message_builders::*, OrderState, Side};

    #[test]
    fn returns_start_messages_event_v50() {
        let mut data = system_event_v50(5000, 'O');
        let mut context = Context::new();
        context.update_clock(0);

        let message = SystemEvent::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.kind(), 'S');
        assert_eq!(*message.nanoseconds(), 5000);
        assert_eq!(*message.event_code(), EventCode::StartMessages);
    }

    #[test]
    fn returns_start_system_event_v41() {
        let mut data = system_event_v41(10000, 'S');
        let mut context = Context::new();
        context.update_clock(30);

        let message = SystemEvent::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'S');
        assert_eq!(*message.nanoseconds(), 30_000_010_000);
        assert_eq!(*message.event_code(), EventCode::StartSystem);
    }

    #[test]
    fn returns_start_market_hours_event() {
        let mut data = system_event_v50(15000, 'Q');
        let mut context = Context::new();
        context.update_clock(0);

        let message = SystemEvent::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::StartMarketHours);
    }

    #[test]
    fn returns_end_market_hours_event() {
        let mut data = system_event_v41(20000, 'M');
        let mut context = Context::new();
        context.update_clock(60);

        let message = SystemEvent::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::EndMarketHours);
    }

    #[test]
    fn returns_end_system_event() {
        let mut data = system_event_v50(25000, 'E');
        let mut context = Context::new();
        context.update_clock(0);

        let message = SystemEvent::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::EndSystem);
    }

    #[test]
    fn returns_end_messages_event() {
        let mut data = system_event_v41(30000, 'C');
        let mut context = Context::new();
        context.update_clock(90);

        let message = SystemEvent::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::EndMessages);
    }

    #[test]
    fn returns_emergency_market_halt_event() {
        let mut data = system_event_v50(35000, 'A');
        let mut context = Context::new();
        context.update_clock(0);

        let message = SystemEvent::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::EmergencyMarketHalt);
    }

    #[test]
    fn returns_emergency_market_quote_only_event() {
        let mut data = system_event_v41(40000, 'R');
        let mut context = Context::new();
        context.update_clock(120);

        let message = SystemEvent::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::EmergencyMarketQuoteOnly);
    }

    #[test]
    fn returns_emergency_market_resumption_event() {
        let mut data = system_event_v50(45000, 'B');
        let mut context = Context::new();
        context.update_clock(0);

        let message = SystemEvent::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.event_code(), EventCode::EmergencyMarketResumption);
    }

    #[test]
    fn handles_both_versions() {
        let mut data_v41 = system_event_v41(1000, 'O');
        let mut data_v50 = system_event_v50(2000, 'O');
        let mut context = Context::new();
        context.update_clock(10);

        let message_v41 = SystemEvent::read(&mut data_v41, &Version::V41, &mut context).unwrap();
        context.update_clock(0);
        let message_v50 = SystemEvent::read(&mut data_v50, &Version::V50, &mut context).unwrap();

        assert_eq!(*message_v41.nanoseconds(), 10_000_001_000);
        assert_eq!(*message_v50.nanoseconds(), 2000);
        assert_eq!(*message_v41.event_code(), EventCode::StartMessages);
        assert_eq!(*message_v50.event_code(), EventCode::StartMessages);
    }

    #[test]
    fn system_event_does_not_modify_context() {
        let mut data = system_event_v50(50000, 'S');
        let mut context = Context::new();
        context.update_clock(0);

        // Add an order to verify it's not modified
        context.active_orders.insert(
            12345,
            OrderState {
                ticker: "TEST".to_string(),
                side: Side::Buy,
                price: 10000,
                shares: 100,
            },
        );

        let _message = SystemEvent::read(&mut data, &Version::V50, &mut context).unwrap();

        // Context should remain unchanged
        assert!(context.active_orders.contains_key(&12345));
        assert_eq!(context.active_orders.len(), 1);
    }
}
