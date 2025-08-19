#[cfg(test)]
pub mod message_builders {
    use crate::message::{OrderState, Side};
    use byteorder::{NetworkEndian, WriteBytesExt};
    use std::io::Cursor;

    // Timestamp helpers
    pub fn timestamp_v41(seconds: u32) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'T');
        data.write_u32::<NetworkEndian>(seconds).unwrap();

        Cursor::new(data)
    }

    // Add Order helpers
    pub fn add_order_v41(
        nanoseconds: u32,
        refno: u64,
        side: Side,
        shares: u32,
        ticker: &str,
        price: u32,
    ) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'A');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();
        data.push(match side {
            Side::Buy => b'B',
            Side::Sell => b'S',
        });
        data.write_u32::<NetworkEndian>(shares).unwrap();
        let mut ticker_bytes = [b' '; 8];
        let ticker_bytes_slice = ticker.as_bytes();
        ticker_bytes[..ticker_bytes_slice.len()].copy_from_slice(ticker_bytes_slice);
        data.extend_from_slice(&ticker_bytes);
        data.write_u32::<NetworkEndian>(price).unwrap();

        Cursor::new(data)
    }

    pub fn add_order_v50(
        nanoseconds: u64,
        refno: u64,
        side: Side,
        shares: u32,
        ticker: &str,
        price: u32,
    ) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'A');
        data.write_u16::<NetworkEndian>(0).unwrap(); // stock locate
        data.write_u16::<NetworkEndian>(0).unwrap(); // tracking number
        data.write_u48::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();
        data.push(match side {
            Side::Buy => b'B',
            Side::Sell => b'S',
        });
        data.write_u32::<NetworkEndian>(shares).unwrap();
        let mut ticker_bytes = [b' '; 8];
        let ticker_bytes_slice = ticker.as_bytes();
        ticker_bytes[..ticker_bytes_slice.len()].copy_from_slice(ticker_bytes_slice);
        data.extend_from_slice(&ticker_bytes);
        data.write_u32::<NetworkEndian>(price).unwrap();

        Cursor::new(data)
    }

    pub fn add_order_with_mpid_v41(
        nanoseconds: u32,
        refno: u64,
        side: Side,
        shares: u32,
        ticker: &str,
        price: u32,
        mpid: &str,
    ) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'F'); // Attribution message type
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();
        data.push(match side {
            Side::Buy => b'B',
            Side::Sell => b'S',
        });
        data.write_u32::<NetworkEndian>(shares).unwrap();
        let mut ticker_bytes = [b' '; 8];
        let ticker_bytes_slice = ticker.as_bytes();
        ticker_bytes[..ticker_bytes_slice.len()].copy_from_slice(ticker_bytes_slice);
        data.extend_from_slice(&ticker_bytes);
        data.write_u32::<NetworkEndian>(price).unwrap();
        let mut mpid_bytes = [b' '; 4];
        let mpid_bytes_slice = mpid.as_bytes();
        mpid_bytes[..mpid_bytes_slice.len()].copy_from_slice(mpid_bytes_slice);
        data.extend_from_slice(&mpid_bytes);

        Cursor::new(data)
    }

    // Cancel Order helpers
    pub fn cancel_order_v41(nanoseconds: u32, refno: u64, shares: u32) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'X');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap(); // nanoseconds
        data.write_u64::<NetworkEndian>(refno).unwrap(); // refno
        data.write_u32::<NetworkEndian>(shares).unwrap(); // shares

        Cursor::new(data)
    }

    pub fn cancel_order_v50(nanoseconds: u64, refno: u64, shares: u32) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'X');
        data.write_u16::<NetworkEndian>(0).unwrap(); // stock locate
        data.write_u16::<NetworkEndian>(0).unwrap(); // tracking number
        data.write_u48::<NetworkEndian>(nanoseconds).unwrap(); // nanoseconds
        data.write_u64::<NetworkEndian>(refno).unwrap(); // refno
        data.write_u32::<NetworkEndian>(shares).unwrap(); // shares

        Cursor::new(data)
    }

    // Delete Order helpers
    pub fn delete_order_v41(nanoseconds: u32, refno: u64) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'D');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();

        Cursor::new(data)
    }

    pub fn delete_order_v50(nanoseconds: u64, refno: u64) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'D');
        data.write_u16::<NetworkEndian>(0).unwrap(); // stock locate
        data.write_u16::<NetworkEndian>(0).unwrap(); // tracking number
        data.write_u48::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();

        Cursor::new(data)
    }

    // Execute Order helpers
    pub fn execute_order_v41(nanoseconds: u32, refno: u64, shares: u32) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'E');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();
        data.write_u32::<NetworkEndian>(shares).unwrap();
        data.write_u64::<NetworkEndian>(12345678).unwrap(); // match number

        Cursor::new(data)
    }

    pub fn execute_order_v50(nanoseconds: u64, refno: u64, shares: u32) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'E');
        data.write_u16::<NetworkEndian>(0).unwrap(); // stock locate
        data.write_u16::<NetworkEndian>(0).unwrap(); // tracking number
        data.write_u48::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();
        data.write_u32::<NetworkEndian>(shares).unwrap();
        data.write_u64::<NetworkEndian>(87654321).unwrap(); // match number

        Cursor::new(data)
    }

    pub fn execute_order_with_price_v41(
        nanoseconds: u32,
        refno: u64,
        shares: u32,
        printable: bool,
        execution_price: u32,
    ) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'C');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(refno).unwrap();
        data.write_u32::<NetworkEndian>(shares).unwrap();
        data.write_u64::<NetworkEndian>(11223344).unwrap(); // match number
        data.push(if printable { b'Y' } else { b'N' });
        data.write_u32::<NetworkEndian>(execution_price).unwrap();

        Cursor::new(data)
    }

    // Replace Order helpers
    pub fn replace_order_v41(
        nanoseconds: u32,
        old_refno: u64,
        new_refno: u64,
        new_shares: u32,
        new_price: u32,
    ) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'U');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(old_refno).unwrap();
        data.write_u64::<NetworkEndian>(new_refno).unwrap();
        data.write_u32::<NetworkEndian>(new_shares).unwrap();
        data.write_u32::<NetworkEndian>(new_price).unwrap();

        Cursor::new(data)
    }

    pub fn replace_order_v50(
        nanoseconds: u64,
        old_refno: u64,
        new_refno: u64,
        new_shares: u32,
        new_price: u32,
    ) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'U');
        data.write_u16::<NetworkEndian>(0).unwrap(); // stock locate
        data.write_u16::<NetworkEndian>(0).unwrap(); // tracking number
        data.write_u48::<NetworkEndian>(nanoseconds).unwrap();
        data.write_u64::<NetworkEndian>(old_refno).unwrap();
        data.write_u64::<NetworkEndian>(new_refno).unwrap();
        data.write_u32::<NetworkEndian>(new_shares).unwrap();
        data.write_u32::<NetworkEndian>(new_price).unwrap();

        Cursor::new(data)
    }

    // System Event helpers
    pub fn system_event_v41(nanoseconds: u32, event_code: char) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'S');
        data.write_u32::<NetworkEndian>(nanoseconds).unwrap();
        data.push(event_code as u8);

        Cursor::new(data)
    }

    pub fn system_event_v50(nanoseconds: u64, event_code: char) -> Cursor<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        data.push(b'S');
        data.write_u16::<NetworkEndian>(0).unwrap(); // stock locate
        data.write_u16::<NetworkEndian>(0).unwrap(); // tracking number
        data.write_u48::<NetworkEndian>(nanoseconds).unwrap();
        data.push(event_code as u8);

        Cursor::new(data)
    }

    // Helper for creating OrderState for context setup
    pub(crate) fn create_order_state(
        ticker: &str,
        side: Side,
        price: u32,
        shares: u32,
    ) -> OrderState {
        OrderState {
            ticker: ticker.to_string(),
            side,
            price,
            shares,
        }
    }

    // Helper for writing message length prefix (useful for reader tests)
    pub fn with_length_prefix(message_cursor: Cursor<Vec<u8>>) -> Cursor<Vec<u8>> {
        let message_data = message_cursor.into_inner();
        let mut data = Vec::<u8>::new();
        data.write_u16::<NetworkEndian>(message_data.len() as u16)
            .unwrap();
        data.extend_from_slice(&message_data);
        Cursor::new(data)
    }

    // Helper for creating multiple messages in sequence (useful for reader tests)
    pub fn create_message_sequence(messages: Vec<Cursor<Vec<u8>>>) -> Cursor<Vec<u8>> {
        let mut combined_data = Vec::<u8>::new();

        for message in messages {
            let prefixed_message = with_length_prefix(message);
            combined_data.extend_from_slice(&prefixed_message.into_inner());
        }

        Cursor::new(combined_data)
    }
}
