use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_kind, read_nanoseconds, read_refno};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct DeleteOrder {
    nanoseconds: u64,
    kind: char,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    from_replace: Option<bool>,
}

impl DeleteOrder {
    pub(crate) fn new(
        nanoseconds: u64,
        kind: char,
        ticker: String,
        side: Side,
        price: u32,
        shares: u32,
        refno: u64,
        from_replace: Option<bool>,
    ) -> Self {
        Self {
            nanoseconds,
            kind,
            ticker,
            side,
            price,
            shares,
            refno,
            from_replace,
        }
    }
}

impl ReadMessage for DeleteOrder {
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

        // Update context
        let order = context
            .active_orders
            .remove(&refno)
            .expect("Order not found");

        // Return message
        Ok(Self {
            nanoseconds,
            kind,
            ticker: order.ticker,
            side: order.side,
            price: order.price,
            shares: order.shares,
            refno,
            from_replace: Some(false),
        })
    }
}

impl IntoOrderMessage for DeleteOrder {
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
            from_replace: self.from_replace,
            mpid: None,
            printable: None,
            execution_price: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::test_helpers::message_builders::*;
    use crate::message::{OrderState, Side};

    #[test]
    fn returns_message_and_removes_from_context_v50() {
        let mut data = delete_order_v50(2000, 98765);
        let mut context = Context::new();
        context.update_clock(0);
        context.active_orders.insert(
            98765,
            create_order_state("NVDA", Side::Sell, 45000, 150),
        );

        let message = DeleteOrder::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.kind(), 'D');
        assert_eq!(*message.nanoseconds(), 2000);
        assert_eq!(*message.refno(), 98765);
        assert_eq!(*message.ticker(), "NVDA");
        assert_eq!(*message.side(), Side::Sell);
        assert_eq!(*message.price(), 45000);
        assert_eq!(*message.shares(), 150);
        assert_eq!(*message.from_replace(), Some(false));

        assert!(!context.active_orders.contains_key(&98765));
    }

    #[test]
    fn returns_message_and_removes_from_context_v41() {
        let mut data = delete_order_v41(3500, 11111);
        let mut context = Context::new();
        context.update_clock(5);
        context.active_orders.insert(
            11111,
            OrderState {
                ticker: "AMD".to_string(),
                side: Side::Buy,
                price: 12000,
                shares: 300,
            },
        );

        let message = DeleteOrder::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'D');
        assert_eq!(*message.nanoseconds(), 5_000_003_500);
        assert_eq!(*message.refno(), 11111);
        assert_eq!(*message.ticker(), "AMD");
        assert_eq!(*message.side(), Side::Buy);
        assert_eq!(*message.price(), 12000);
        assert_eq!(*message.shares(), 300);
        assert_eq!(*message.from_replace(), Some(false));

        assert!(!context.active_orders.contains_key(&11111));
    }

    #[test]
    #[should_panic(expected = "Order not found")]
    fn panics_when_order_not_found() {
        let mut data = delete_order_v50(1000, 99999);
        let mut context = Context::new();
        context.update_clock(0);

        DeleteOrder::read(&mut data, &Version::V50, &mut context).unwrap();
    }

    #[test]
    fn into_order_message_conversion() {
        let delete_order = DeleteOrder {
            nanoseconds: 5000,
            kind: 'D',
            ticker: "META".to_string(),
            side: Side::Buy,
            price: 32000,
            shares: 80,
            refno: 77777,
            from_replace: Some(true),
        };

        let order_message = delete_order.into_order_message("2023-12-25".to_string());

        assert_eq!(*order_message.date(), "2023-12-25");
        assert_eq!(order_message.nanoseconds, 5000);
        assert_eq!(order_message.kind, 'D');
        assert_eq!(order_message.ticker, "META");
        assert_eq!(order_message.side, Side::Buy);
        assert_eq!(order_message.price, 32000);
        assert_eq!(order_message.shares, 80);
        assert_eq!(order_message.refno, 77777);
        assert_eq!(order_message.from_replace, Some(true));
        assert_eq!(order_message.mpid, None);
        assert_eq!(order_message.printable, None);
        assert_eq!(order_message.execution_price, None);
    }

    #[test]
    fn creates_delete_order_with_new_method() {
        let delete_order = DeleteOrder::new(
            1500,
            'D',
            "GOOGL".to_string(),
            Side::Sell,
            275000,
            25,
            88888,
            Some(true),
        );

        assert_eq!(delete_order.nanoseconds, 1500);
        assert_eq!(delete_order.kind, 'D');
        assert_eq!(delete_order.ticker, "GOOGL");
        assert_eq!(delete_order.side, Side::Sell);
        assert_eq!(delete_order.price, 275000);
        assert_eq!(delete_order.shares, 25);
        assert_eq!(delete_order.refno, 88888);
        assert_eq!(delete_order.from_replace, Some(true));
    }
}
