use std::io::{Read, Result, Seek, SeekFrom};

use super::{
    read_kind, read_nanoseconds, read_price, read_refno, read_shares, AddOrder, Context,
    DeleteOrder, Version,
};

pub(crate) fn read_replace_order<T>(
    buffer: &mut T,
    version: &Version,
    context: &mut Context,
) -> Result<(DeleteOrder, AddOrder)>
where
    T: Read + Seek,
{
    // Read data from buffer
    let _kind = read_kind(buffer)?;
    if version == &Version::V50 {
        buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
    }
    let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
    let old_refno = read_refno(buffer)?;
    let new_refno = read_refno(buffer)?;
    let new_shares = read_shares(buffer)?;
    let new_price = read_price(buffer)?;

    // Update context
    let mut order = context
        .active_orders
        .remove(&old_refno)
        .expect("Order not found");
    let ticker = order.ticker.clone();
    let side = order.side;
    let old_price = order.price;
    let old_shares = order.shares;
    order.price = new_price;
    order.shares = new_shares;
    context.active_orders.insert(new_refno, order);

    // Split the replacement order into delete and add parts
    let delete_order = DeleteOrder::new(
        nanoseconds,
        'D', // `kind`
        ticker.clone(),
        side,
        old_price,
        old_shares,
        old_refno,
        Some(true), // `from_replace`
    );
    let add_order = AddOrder::new(
        nanoseconds,
        'A', // `kind`
        ticker.clone(),
        side,
        new_price,
        new_shares,
        new_refno,
        Some(true), // `from_replace`
        None,       // `mpid`
    );

    // Return messages
    Ok((delete_order, add_order))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{test_helpers::message_builders::*, OrderState, Side};

    #[test]
    fn returns_delete_and_add_orders_v50() {
        let mut data = replace_order_v50(9000, 44444, 55555, 150, 28000);
        let mut context = Context::new();
        context.update_clock(0);
        context.active_orders.insert(
            44444,
            OrderState {
                ticker: "PYPL".to_string(),
                side: Side::Buy,
                price: 25000,
                shares: 100,
            },
        );

        let (delete_order, add_order) =
            read_replace_order(&mut data, &Version::V50, &mut context).unwrap();

        // Check delete order
        assert_eq!(*delete_order.kind(), 'D');
        assert_eq!(*delete_order.nanoseconds(), 9000);
        assert_eq!(*delete_order.refno(), 44444);
        assert_eq!(*delete_order.ticker(), "PYPL");
        assert_eq!(*delete_order.side(), Side::Buy);
        assert_eq!(*delete_order.price(), 25000);
        assert_eq!(*delete_order.shares(), 100);
        assert_eq!(*delete_order.from_replace(), Some(true));

        // Check add order
        assert_eq!(*add_order.kind(), 'A');
        assert_eq!(*add_order.nanoseconds(), 9000);
        assert_eq!(*add_order.refno(), 55555);
        assert_eq!(*add_order.ticker(), "PYPL");
        assert_eq!(*add_order.side(), Side::Buy);
        assert_eq!(*add_order.price(), 28000);
        assert_eq!(*add_order.shares(), 150);
        assert_eq!(*add_order.from_replace(), Some(true));
        assert_eq!(*add_order.mpid(), None);

        // Check context updates
        assert!(!context.active_orders.contains_key(&44444));
        assert!(context.active_orders.contains_key(&55555));
        let new_order = &context.active_orders[&55555];
        assert_eq!(new_order.ticker, "PYPL");
        assert_eq!(new_order.side, Side::Buy);
        assert_eq!(new_order.price, 28000);
        assert_eq!(new_order.shares, 150);
    }

    #[test]
    fn returns_delete_and_add_orders_v41() {
        let mut data = replace_order_v41(12000, 11111, 22222, 80, 18500);
        let mut context = Context::new();
        context.update_clock(25);
        context.active_orders.insert(
            11111,
            OrderState {
                ticker: "SQ".to_string(),
                side: Side::Sell,
                price: 17000,
                shares: 120,
            },
        );

        let (delete_order, add_order) =
            read_replace_order(&mut data, &Version::V41, &mut context).unwrap();

        // Check delete order
        assert_eq!(*delete_order.kind(), 'D');
        assert_eq!(*delete_order.nanoseconds(), 25_000_012_000);
        assert_eq!(*delete_order.refno(), 11111);
        assert_eq!(*delete_order.ticker(), "SQ");
        assert_eq!(*delete_order.side(), Side::Sell);
        assert_eq!(*delete_order.price(), 17000);
        assert_eq!(*delete_order.shares(), 120);
        assert_eq!(*delete_order.from_replace(), Some(true));

        // Check add order
        assert_eq!(*add_order.kind(), 'A');
        assert_eq!(*add_order.nanoseconds(), 25_000_012_000);
        assert_eq!(*add_order.refno(), 22222);
        assert_eq!(*add_order.ticker(), "SQ");
        assert_eq!(*add_order.side(), Side::Sell);
        assert_eq!(*add_order.price(), 18500);
        assert_eq!(*add_order.shares(), 80);
        assert_eq!(*add_order.from_replace(), Some(true));
        assert_eq!(*add_order.mpid(), None);

        // Check context updates
        assert!(!context.active_orders.contains_key(&11111));
        assert!(context.active_orders.contains_key(&22222));
        let new_order = &context.active_orders[&22222];
        assert_eq!(new_order.ticker, "SQ");
        assert_eq!(new_order.side, Side::Sell);
        assert_eq!(new_order.price, 18500);
        assert_eq!(new_order.shares, 80);
    }

    #[test]
    #[should_panic(expected = "Order not found")]
    fn panics_when_order_not_found() {
        let mut data = replace_order_v50(5000, 99999, 88888, 50, 10000);
        let mut context = Context::new();
        context.update_clock(0);

        read_replace_order(&mut data, &Version::V50, &mut context).unwrap();
    }

    #[test]
    fn preserves_original_order_info() {
        let mut data = replace_order_v41(3000, 33333, 44444, 200, 35000);
        let mut context = Context::new();
        context.update_clock(30);
        context.active_orders.insert(
            33333,
            OrderState {
                ticker: "ROKU".to_string(),
                side: Side::Buy,
                price: 30000,
                shares: 175,
            },
        );

        let (delete_order, add_order) =
            read_replace_order(&mut data, &Version::V41, &mut context).unwrap();

        // Both orders should have same ticker and side as original
        assert_eq!(*delete_order.ticker(), "ROKU");
        assert_eq!(*delete_order.side(), Side::Buy);
        assert_eq!(*add_order.ticker(), "ROKU");
        assert_eq!(*add_order.side(), Side::Buy);

        // Delete order should have original price/shares, add order should have new ones
        assert_eq!(*delete_order.price(), 30000);
        assert_eq!(*delete_order.shares(), 175);
        assert_eq!(*add_order.price(), 35000);
        assert_eq!(*add_order.shares(), 200);
    }

    #[test]
    fn replaces_same_refno_with_different_attributes() {
        let mut data = replace_order_v50(7500, 66666, 66666, 300, 42000);
        let mut context = Context::new();
        context.update_clock(0);
        context.active_orders.insert(
            66666,
            OrderState {
                ticker: "TWTR".to_string(),
                side: Side::Sell,
                price: 40000,
                shares: 250,
            },
        );

        let (delete_order, add_order) =
            read_replace_order(&mut data, &Version::V50, &mut context).unwrap();

        // Same refno but different attributes
        assert_eq!(*delete_order.refno(), 66666);
        assert_eq!(*add_order.refno(), 66666);

        assert_eq!(*delete_order.price(), 40000);
        assert_eq!(*delete_order.shares(), 250);
        assert_eq!(*add_order.price(), 42000);
        assert_eq!(*add_order.shares(), 300);

        // Context should still have the order with updated attributes
        assert!(context.active_orders.contains_key(&66666));
        let updated_order = &context.active_orders[&66666];
        assert_eq!(updated_order.price, 42000);
        assert_eq!(updated_order.shares, 300);
    }

    #[test]
    fn both_orders_marked_as_from_replace() {
        let mut data = replace_order_v41(15000, 77777, 88888, 90, 52000);
        let mut context = Context::new();
        context.update_clock(45);
        context.active_orders.insert(
            77777,
            OrderState {
                ticker: "DOCU".to_string(),
                side: Side::Buy,
                price: 50000,
                shares: 125,
            },
        );

        let (delete_order, add_order) =
            read_replace_order(&mut data, &Version::V41, &mut context).unwrap();

        assert_eq!(*delete_order.from_replace(), Some(true));
        assert_eq!(*add_order.from_replace(), Some(true));
    }
}
