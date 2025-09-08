use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{
    read_kind, read_nanoseconds, read_refno, read_shares, Context, IntoOrderMessage, OrderMessage,
    ReadMessage, Side, Version,
};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct CancelOrder {
    nanoseconds: u64,
    kind: char,
    pub(crate) ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    pub(crate) refno: u64,
}

impl ReadMessage for CancelOrder {
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
        let refno = read_refno(buffer)?;
        let shares = read_shares(buffer)?;

        // Update context
        let order = context
            .active_orders
            .get_mut(&refno)
            .expect("Order not found");
        order.shares -= shares;

        // Return message
        Ok(Self {
            nanoseconds,
            kind,
            ticker: order.ticker.clone(),
            side: order.side,
            price: order.price,
            shares,
            refno,
        })
    }
}

impl IntoOrderMessage for CancelOrder {
    fn into_order_message(self, date: String) -> OrderMessage {
        OrderMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            ticker: self.ticker,
            side: self.side,
            price: self.price,
            shares: self.shares,
            refno: self.refno,
            from_replace: None,
            mpid: None,
            printable: None,
            execution_price: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{test_helpers::message_builders::*, Side};

    #[test]
    fn returns_message_and_updates_shares_v50() {
        let mut data = cancel_order_v50(0, 0, 10);
        let mut context = Context::new();
        context.update_clock(0);
        context
            .active_orders
            .insert(0, create_order_state("A", Side::Buy, 0, 100));
        let message = CancelOrder::read(&mut data, &Version::V50, &mut context).unwrap();
        assert_eq!(*message.kind(), 'X');
        assert_eq!(context.active_orders[&0].shares, 90);
    }

    #[test]
    fn returns_message_and_updates_shares_v41() {
        let mut data = cancel_order_v41(0, 0, 10);
        let mut context = Context::new();
        context.update_clock(0);
        context
            .active_orders
            .insert(0, create_order_state("A", Side::Buy, 0, 100));
        let message = CancelOrder::read(&mut data, &Version::V41, &mut context).unwrap();
        assert_eq!(*message.kind(), 'X');
        assert_eq!(context.active_orders[&0].shares, 90);
    }

    #[test]
    #[should_panic(expected = "Order not found")]
    fn panics_with_message() {
        let mut data = cancel_order_v41(0, 0, 10);
        let mut context = Context::new();
        context.update_clock(0);
        let _ = CancelOrder::read(&mut data, &Version::V41, &mut context).unwrap();
    }
}
