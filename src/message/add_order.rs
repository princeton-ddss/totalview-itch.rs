use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{
    read_kind, read_mpid, read_nanoseconds, read_price, read_refno, read_shares, read_side,
    read_ticker,
};
use super::{Context, OrderState, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct AddOrder {
    nanoseconds: u64,
    kind: char,
    pub(crate) ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    from_replace: Option<bool>,
    mpid: Option<String>,
}

impl AddOrder {
    pub(crate) fn new(
        nanoseconds: u64,
        kind: char,
        ticker: String,
        side: Side,
        price: u32,
        shares: u32,
        refno: u64,
        from_replace: Option<bool>,
        mpid: Option<String>,
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
            mpid,
        }
    }
}

impl ReadMessage for AddOrder {
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
        let side = read_side(buffer)?;
        let shares = read_shares(buffer)?;
        let ticker = read_ticker(buffer)?;
        let price = read_price(buffer)?;
        let mpid = if kind == 'F' {
            Some(read_mpid(buffer)?)
        } else {
            None
        };

        // Update context
        let order = OrderState {
            ticker: ticker.clone(),
            side,
            price,
            shares,
        };
        context.active_orders.insert(refno, order);

        // Return message
        Ok(Self {
            nanoseconds,
            kind,
            ticker,
            side,
            price,
            shares,
            refno,
            from_replace: Some(false),
            mpid,
        })
    }
}

impl IntoOrderMessage for AddOrder {
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
            mpid: self.mpid,
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
    fn returns_message_and_updates_context_v50() {
        let mut data = add_order_v50(1000, 12345, Side::Buy, 100, "AAPL", 15000);
        let mut context = Context::new();
        context.update_clock(0);

        let message = AddOrder::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.kind(), 'A');
        assert_eq!(*message.nanoseconds(), 1000);
        assert_eq!(*message.refno(), 12345);
        assert_eq!(*message.side(), Side::Buy);
        assert_eq!(*message.shares(), 100);
        assert_eq!(*message.ticker(), "AAPL");
        assert_eq!(*message.price(), 15000);
        assert_eq!(*message.from_replace(), Some(false));
        assert_eq!(*message.mpid(), None);

        let order = &context.active_orders[&12345];
        assert_eq!(order.ticker, "AAPL");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 15000);
        assert_eq!(order.shares, 100);
    }

    #[test]
    fn returns_message_and_updates_context_v41() {
        let mut data = add_order_v41(1000, 12345, Side::Sell, 200, "TSLA", 25000);
        let mut context = Context::new();
        context.update_clock(1);

        let message = AddOrder::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'A');
        assert_eq!(*message.nanoseconds(), 1_000_001_000);
        assert_eq!(*message.refno(), 12345);
        assert_eq!(*message.side(), Side::Sell);
        assert_eq!(*message.shares(), 200);
        assert_eq!(*message.ticker(), "TSLA");
        assert_eq!(*message.price(), 25000);
        assert_eq!(*message.from_replace(), Some(false));
        assert_eq!(*message.mpid(), None);

        let order = &context.active_orders[&12345];
        assert_eq!(order.ticker, "TSLA");
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.price, 25000);
        assert_eq!(order.shares, 200);
    }

    #[test]
    fn handles_attribution_message_with_mpid() {
        let mut data = add_order_with_mpid_v41(2000, 54321, Side::Buy, 50, "MSFT", 30000, "NSDQ");
        let mut context = Context::new();
        context.update_clock(2);

        let message = AddOrder::read(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*message.kind(), 'F');
        assert_eq!(*message.nanoseconds(), 2_000_002_000);
        assert_eq!(*message.refno(), 54321);
        assert_eq!(*message.side(), Side::Buy);
        assert_eq!(*message.shares(), 50);
        assert_eq!(*message.ticker(), "MSFT");
        assert_eq!(*message.price(), 30000);
        assert_eq!(*message.from_replace(), Some(false));
        assert_eq!(*message.mpid(), Some("NSDQ".to_string()));

        let order = &context.active_orders[&54321];
        assert_eq!(order.ticker, "MSFT");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 30000);
        assert_eq!(order.shares, 50);
    }

    #[test]
    fn into_order_message_conversion() {
        let add_order = AddOrder {
            nanoseconds: 1000,
            kind: 'A',
            ticker: "GOOG".to_string(),
            side: Side::Buy,
            price: 280000,
            shares: 75,
            refno: 99999,
            from_replace: Some(false),
            mpid: None,
        };

        let order_message = add_order.into_order_message("2023-01-15".to_string());

        assert_eq!(*order_message.date(), "2023-01-15");
        assert_eq!(order_message.nanoseconds, 1000);
        assert_eq!(order_message.kind, 'A');
        assert_eq!(order_message.ticker, "GOOG");
        assert_eq!(order_message.side, Side::Buy);
        assert_eq!(order_message.price, 280000);
        assert_eq!(order_message.shares, 75);
        assert_eq!(order_message.refno, 99999);
        assert_eq!(order_message.from_replace, Some(false));
        assert_eq!(order_message.mpid, None);
        assert_eq!(order_message.printable, None);
        assert_eq!(order_message.execution_price, None);
    }

    #[test]
    fn trims_ticker_whitespace() {
        let mut data = add_order_v50(500, 11111, Side::Buy, 25, "IBM     ", 12500);
        let mut context = Context::new();
        context.update_clock(0);

        let message = AddOrder::read(&mut data, &Version::V50, &mut context).unwrap();

        assert_eq!(*message.ticker(), "IBM");
        assert_eq!(context.active_orders[&11111].ticker, "IBM");
    }
}
