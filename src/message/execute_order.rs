use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{
    read_kind, read_nanoseconds, read_price, read_printable, read_refno, read_shares, Context,
    IntoOrderMessage, OrderMessage, ReadMessage, Side, Version,
};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ExecuteOrder {
    nanoseconds: u64,
    kind: char,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    printable: Option<bool>,
    execution_price: Option<u32>,
}

impl ReadMessage for ExecuteOrder {
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
        buffer.seek(SeekFrom::Current(8))?; // Discard match number
        let (printable, execution_price) = if kind == 'C' {
            let printable = Some(read_printable(buffer)?);
            let execution_price = Some(read_price(buffer)?);
            (printable, execution_price)
        } else {
            (None, None)
        };

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
            printable,
            execution_price,
        })
    }
}

impl IntoOrderMessage for ExecuteOrder {
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
            printable: self.printable,
            execution_price: self.execution_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{test_helpers::message_builders::*, OrderState, Side};

    #[test]
    fn returns_message_and_updates_shares_v50() {
        let mut data = execute_order_v50(3000, 55555, 50);
        let mut context = Context::new();
        context.update_clock(0);
        context.active_orders.insert(
            55555,
            OrderState {
                ticker: "INTC".to_string(),
                side: Side::Buy,
                price: 5500,
                shares: 200,
            },
        );

        let message = ExecuteOrder::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.kind(), 'E');
        assert_eq!(*message.nanoseconds(), 3000);
        assert_eq!(*message.refno(), 55555);
        assert_eq!(*message.ticker(), "INTC");
        assert_eq!(*message.side(), Side::Buy);
        assert_eq!(*message.price(), 5500);
        assert_eq!(*message.shares(), 50);
        assert_eq!(*message.printable(), None);
        assert_eq!(*message.execution_price(), None);

        let order = &context.active_orders[&55555];
        assert_eq!(order.shares, 150); // 200 - 50
    }

    #[test]
    fn returns_message_and_updates_shares_v41() {
        let mut data = execute_order_v41(4500, 33333, 75);
        let mut context = Context::new();
        context.update_clock(10);
        context.active_orders.insert(
            33333,
            OrderState {
                ticker: "ORCL".to_string(),
                side: Side::Sell,
                price: 8200,
                shares: 300,
            },
        );

        let message = ExecuteOrder::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'E');
        assert_eq!(*message.nanoseconds(), 10_000_004_500);
        assert_eq!(*message.refno(), 33333);
        assert_eq!(*message.ticker(), "ORCL");
        assert_eq!(*message.side(), Side::Sell);
        assert_eq!(*message.price(), 8200);
        assert_eq!(*message.shares(), 75);
        assert_eq!(*message.printable(), None);
        assert_eq!(*message.execution_price(), None);

        let order = &context.active_orders[&33333];
        assert_eq!(order.shares, 225); // 300 - 75
    }

    #[test]
    fn handles_execute_with_price_message() {
        let mut data = execute_order_with_price_v41(6000, 77777, 100, true, 15500);
        let mut context = Context::new();
        context.update_clock(15);
        context.active_orders.insert(
            77777,
            OrderState {
                ticker: "NFLX".to_string(),
                side: Side::Buy,
                price: 15000,
                shares: 250,
            },
        );

        let message = ExecuteOrder::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'C');
        assert_eq!(*message.nanoseconds(), 15_000_006_000);
        assert_eq!(*message.refno(), 77777);
        assert_eq!(*message.ticker(), "NFLX");
        assert_eq!(*message.side(), Side::Buy);
        assert_eq!(*message.price(), 15000);
        assert_eq!(*message.shares(), 100);
        assert_eq!(*message.printable(), Some(true));
        assert_eq!(*message.execution_price(), Some(15500));

        let order = &context.active_orders[&77777];
        assert_eq!(order.shares, 150); // 250 - 100
    }

    #[test]
    fn handles_non_printable_execute_with_price() {
        let mut data = execute_order_with_price_v41(7000, 88888, 25, false, 22500);
        let mut context = Context::new();
        context.update_clock(20);
        context.active_orders.insert(
            88888,
            OrderState {
                ticker: "UBER".to_string(),
                side: Side::Sell,
                price: 22000,
                shares: 100,
            },
        );

        let message = ExecuteOrder::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'C');
        assert_eq!(*message.printable(), Some(false));
        assert_eq!(*message.execution_price(), Some(22500));

        let order = &context.active_orders[&88888];
        assert_eq!(order.shares, 75); // 100 - 25
    }

    #[test]
    #[should_panic(expected = "Order not found")]
    fn panics_when_order_not_found() {
        let mut data = execute_order_v50(1000, 99999, 50);
        let mut context = Context::new();
        context.update_clock(0);

        ExecuteOrder::read(&mut data, &Version::V50, &mut context).unwrap();
    }

    #[test]
    fn into_order_message_conversion() {
        let execute_order = ExecuteOrder {
            nanoseconds: 8000,
            kind: 'C',
            ticker: "SNAP".to_string(),
            side: Side::Buy,
            price: 1200,
            shares: 40,
            refno: 66666,
            printable: Some(true),
            execution_price: Some(1250),
        };

        let order_message = execute_order.into_order_message("2024-03-15".to_string());

        assert_eq!(*order_message.date(), "2024-03-15");
        assert_eq!(order_message.nanoseconds, 8000);
        assert_eq!(order_message.kind, 'C');
        assert_eq!(order_message.ticker, "SNAP");
        assert_eq!(order_message.side, Side::Buy);
        assert_eq!(order_message.price, 1200);
        assert_eq!(order_message.shares, 40);
        assert_eq!(order_message.refno, 66666);
        assert_eq!(order_message.from_replace, None);
        assert_eq!(order_message.mpid, None);
        assert_eq!(order_message.printable, Some(true));
        assert_eq!(order_message.execution_price, Some(1250));
    }

    #[test]
    fn reduces_order_shares_correctly() {
        let mut data = execute_order_v50(5000, 12321, 175);
        let mut context = Context::new();
        context.update_clock(0);
        context.active_orders.insert(
            12321,
            OrderState {
                ticker: "SPOT".to_string(),
                side: Side::Buy,
                price: 18000,
                shares: 400,
            },
        );

        assert_eq!(context.active_orders[&12321].shares, 400);

        let _message = ExecuteOrder::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(context.active_orders[&12321].shares, 225); // 400 - 175
    }
}
