use std::io::{Read, Result, Seek, SeekFrom};

use super::{read_nanoseconds, read_price, read_refno, read_shares};
use super::{AddOrder, Context, DeleteOrder, Version};

pub(crate) fn read_replace_order<T>(
    buffer: &mut T,
    version: &Version,
    context: &mut Context,
) -> Result<(DeleteOrder, AddOrder)>
where
    T: Read + Seek,
{
    if version == &Version::V50 {
        buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
    }

    // Read data from buffer
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
    let side = order.side;
    let ticker = order.ticker.clone();
    let old_shares = order.shares;
    let old_price = order.price;
    order.shares = new_shares;
    order.price = new_price;
    context.active_orders.insert(new_refno, order);

    // Split the replacement order into delete and add parts
    let delete_order = DeleteOrder::new(
        nanoseconds,
        old_refno,
        ticker.clone(),
        side,
        old_price,
        old_shares,
    );
    let add_order = AddOrder::new(
        nanoseconds,
        new_refno,
        ticker.clone(),
        side,
        new_price,
        new_shares,
    );

    // Return messages
    Ok((delete_order, add_order))
}
