use crate::message::{AddOrderMessage, ModifyOrderMessage};
use std::collections::HashMap;
// use std::collections::BTreeMap;

pub struct Order {
    refno: u64,
    ticker: String,
    // side: Side,
    side: char,
    shares: u32,
    price: u32,
}

impl Order {
    pub fn new(refno: u64, ticker: String, side: char, shares: u32, price: u32) -> Self {
        Self { refno, ticker, side, shares, price }
    }

    fn get_shares(&self) -> u32 {
        self.shares
    }

    fn decrease_shares(&mut self, shares: u32) {
        self.shares -= shares;
    }
}

pub struct OrderList {
    list: HashMap<u64,Order>,
}

impl OrderList {
    fn add<T: AddOrderMessage>(&mut self, message: T) {
        let order = message.to_order();
        let refno = message.get_refno();
        self.list.insert(refno, order);
    }

    fn modify<T: ModifyOrderMessage>(&mut self, message: T) {
        let refno = message.get_refno();
        let shares = message.get_shares();
        self.list.entry(refno).and_modify(|s| s.decrease_shares(shares));

        let order = self.list.get(&refno).unwrap();
        if order.get_shares() == 0 {
            self.list.remove(&refno);
        }
    }
}

struct OrderBook {
    bids: HashMap<u32, u32>,
    asks: HashMap<u32, u32>,
    nlevels: u32,
}

impl OrderBook {
    fn add<T: AddOrderMessage>(&mut self, message: T) {
        match message.get_side() {
            Buy => {
                let price = message.get_price();
                let shares = message.get_shares();
                self.bids.entry(price).or_insert(shares);
            },
            Sell => {
                let price = message.get_price();
                let shares = message.get_shares();
                self.asks.entry(price).or_insert(shares);
            }
        }
    }

    fn modify<T: ModifyOrderMessage>(&mut self, message: T) {
        match message.get_side() {
            Buy => {
                let price = message.get_price();
                let shares = message.get_shares();
                // TODO: what if price key is missing?
                self.bids.entry(price).and_modify(|s| *s -= shares);
                // TODO: what if this is < 0?
                if *self.bids.get(&price).unwrap() == 0 {
                    self.bids.remove(&price);
                }
            },
            Sell => {
                let price = message.get_price();
                let shares = message.get_shares();
                // TODO: what if price key is missing?
                self.asks.entry(price).and_modify(|s| *s -= shares);
                // TODO: what if this is < 0?
                if *self.asks.get(&price).unwrap() == 0 {
                    self.asks.remove(&price);
                }
            }
        }
    }

    fn write_line(&self) {
        // zip prices and shares
        // sort by prices (desc for bids; ascend for asks)
        // write prices, then shares
        // "p1, p2, ..., pN, s1, s2, ..., sN"
    }
}