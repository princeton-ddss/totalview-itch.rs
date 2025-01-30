use std::collections::BTreeMap;

enum Side {
    Buy,
    Sell
}

pub struct Order {
    refno: u64,
    ticker: String,
    side: Side,
    shares: u32,
    price: u32,
}

impl Order {
    
    pub fn new(refno: u64, ticker: String, side: Side, shares: u32, price: u32) -> Self {
        Self { refno, ticker, side, shares, price }
    }

    fn get_shares(&self) -> u32 {
        self.shares
    }

    fn decrease_shares(&self, shares: u32) {
        self.shares -= shares;
    }
}

enum CreateOrderMessage {
    AddOrderMessage,
    AddOrderWithMPIDMessage
}

enum ModifyOrderMessage {
    ExecuteOrderMessage,
    ExecuteOrderWithPriceMessage,
    CancelOrderMessage,
    DeleteOrderMessage,
}

pub struct OrderList {
    list: HashMap<u64, Order>,
}

impl OrderList {

    fn add(&mut self, message: CreateOrderMessage) {
        let order = message.to_order();
        let refno = message.get_refno();
        // TODO: what is insert's behavior if the key exists?
        self.list.insert(refno, order);
    }

    fn modify(&mut self, message: ModifyOrderMessage) {
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
    bids: BTreeMap<u32, u32>,
    asks: BTreeMap<u32, u32>,
    nlevels: u32,
}

impl OrderBook {
    
    fn add(&mut self, message: CreateOrderMessage) {
        match message.get_side() {
            Buy => {

            },
            Sell => {

            }
        }
    }

    fn modify(&self, message: ModifyOrderMessage) {
        match message.get_side() {
            Buy => {

            },
            Sell => {

            },
        }
    }

    fn to_csv(&self) -> String {
        for (price, shares) in self.bids.iter().rev() {

        }
        for (price, shares) in self.asks.iter() {

        }
    }
}

#[test]
fn main() {
    let orderbook = OrderBook::new();
    let add = AddMessage::new();
    
}