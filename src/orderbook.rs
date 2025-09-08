use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};

use serde::Serialize;

use crate::message::Side;

#[derive(Debug, Serialize)]
pub struct OrderBookSnapshot {
    pub date: String,
    pub ticker: String,
    pub timestamp: u64,
    pub data: Vec<i64>, /* [bid_price_1, bid_size_1, bid_price_2, bid_size_2, ..., ask_price_1,
                         * ask_size_1, ...] */
}

pub struct OrderBook {
    date: String,
    ticker: String,
    timestamp: u64,
    levels: usize,
    bids: HashMap<u32, u32>, // price -> total_shares
    asks: HashMap<u32, u32>, // price -> total_shares
}

impl OrderBook {
    pub fn new(date: String, ticker: String, levels: usize) -> Self {
        Self {
            date,
            ticker,
            timestamp: 0,
            levels,
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    pub fn ticker(&self) -> &str {
        &self.ticker
    }

    pub fn bids(&self) -> &HashMap<u32, u32> {
        &self.bids
    }

    pub fn asks(&self) -> &HashMap<u32, u32> {
        &self.asks
    }

    /// Get top N bid levels
    pub fn top_bids(&self, n: usize) -> Vec<(u32, u32)> {
        if self.bids.is_empty() {
            return Vec::new();
        }

        let mut bids: Vec<(u32, u32)> = self
            .bids
            .iter()
            .map(|(&price, &shares)| (price, shares))
            .collect();

        // Use partial_sort for better performance when n << bids.len()
        let take = n.min(bids.len());
        bids.select_nth_unstable_by(take.saturating_sub(1), |a, b| b.0.cmp(&a.0));
        bids.truncate(take);
        bids.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        bids
    }

    /// Get top N ask levels
    pub fn top_asks(&self, n: usize) -> Vec<(u32, u32)> {
        if self.asks.is_empty() {
            return Vec::new();
        }

        let mut asks: Vec<(u32, u32)> = self
            .asks
            .iter()
            .map(|(&price, &shares)| (price, shares))
            .collect();

        // Use partial_sort for better performance when n << asks.len()
        let take = n.min(asks.len());
        asks.select_nth_unstable_by(take.saturating_sub(1), |a, b| a.0.cmp(&b.0));
        asks.truncate(take);
        asks.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        asks
    }

    /// Create a snapshot of the order book with top N levels on each side
    pub fn snapshot(&self) -> OrderBookSnapshot {
        // Pre-allocate with exact capacity
        let mut data = Vec::with_capacity(self.levels * 4);

        // Get actual bid/ask data
        let bids = self.top_bids(self.levels);
        let asks = self.top_asks(self.levels);

        // Add bid levels with padding - use iterators for better performance
        for i in 0..self.levels {
            let (price, size) = bids.get(i).copied().unwrap_or((0, 0));
            data.push(if price == 0 { -1 } else { price as i64 });
            data.push(if size == 0 { -1 } else { size as i64 });
        }

        // Add ask levels with padding
        for i in 0..self.levels {
            let (price, size) = asks.get(i).copied().unwrap_or((0, 0));
            data.push(if price == 0 { -1 } else { price as i64 });
            data.push(if size == 0 { -1 } else { size as i64 });
        }

        OrderBookSnapshot {
            date: self.date.clone(),
            ticker: self.ticker.clone(),
            timestamp: self.timestamp,
            data,
        }
    }

    /// Add shares to a price level
    pub fn add_order(&mut self, side: Side, price: u32, shares: u32, timestamp: u64) {
        self.timestamp = timestamp;
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        *book.entry(price).or_insert(0) += shares;
    }

    /// Remove shares from a price level
    pub fn remove_order(
        &mut self,
        side: Side,
        price: u32,
        shares: u32,
        timestamp: u64,
    ) -> Result<()> {
        self.timestamp = timestamp;
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        match book.get_mut(&price) {
            Some(current_shares) => {
                if *current_shares < shares {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "Cannot remove {} shares from price level {} (only {} available)",
                            shares, price, current_shares
                        ),
                    ));
                }

                if *current_shares == shares {
                    book.remove(&price);
                } else {
                    *current_shares -= shares;
                }
                Ok(())
            }
            None => Err(Error::new(
                ErrorKind::NotFound,
                format!("No orders found at price level {}", price),
            )),
        }
    }

    /// Execute shares at a price level (reduces volume)
    pub fn execute_order(
        &mut self,
        side: Side,
        price: u32,
        executed_shares: u32,
        timestamp: u64,
    ) -> Result<()> {
        self.timestamp = timestamp;
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        match book.get_mut(&price) {
            Some(current_shares) => {
                if *current_shares < executed_shares {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "Cannot execute {} shares from price level {} (only {} available)",
                            executed_shares, price, current_shares
                        ),
                    ));
                }

                if *current_shares == executed_shares {
                    book.remove(&price);
                } else {
                    *current_shares -= executed_shares;
                }
                Ok(())
            }
            None => Err(Error::new(
                ErrorKind::NotFound,
                format!("No orders found at price level {}", price),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_shares() {
        let mut book = OrderBook::new("01/01/2025".to_string(), "XYZ".to_string(), 5);

        book.add_order(Side::Buy, 1000, 100, 0);
        assert!(book.bids().get(&1000).is_some());
        assert_eq!(*book.bids().get(&1000).unwrap(), 100);

        book.add_order(Side::Buy, 1000, 500, 1);
        assert!(book.bids().get(&1000).is_some());
        assert_eq!(*book.bids().get(&1000).unwrap(), 600);

        book.add_order(Side::Sell, 1100, 200, 2);
        assert!(book.asks().get(&1100).is_some());
        assert_eq!(*book.asks().get(&1100).unwrap(), 200);

        book.add_order(Side::Sell, 1200, 100, 3);
        assert!(book.asks().get(&1200).is_some());
        assert_eq!(*book.asks().get(&1200).unwrap(), 100);
        assert_eq!(book.timestamp, 3);
    }

    #[test]
    fn removes_shares() {
        let mut book = OrderBook::new("01/01/2025".to_string(), "XYZ".to_string(), 5);

        book.add_order(Side::Buy, 1000, 100, 0);
        assert!(book.bids().get(&1000).is_some());
        assert_eq!(*book.bids().get(&1000).unwrap(), 100);

        book.remove_order(Side::Buy, 1000, 50, 1).unwrap();
        assert!(book.bids().get(&1000).is_some());
        assert_eq!(*book.bids().get(&1000).unwrap(), 50);

        book.remove_order(Side::Buy, 1000, 50, 2).unwrap();
        assert!(book.bids().get(&1000).is_none());

        book.add_order(Side::Sell, 1100, 100, 3);
        assert!(book.asks().get(&1100).is_some());
        assert_eq!(*book.asks().get(&1100).unwrap(), 100);

        book.remove_order(Side::Sell, 1100, 50, 4).unwrap();
        assert!(book.asks().get(&1100).is_some());
        assert_eq!(*book.asks().get(&1100).unwrap(), 50);

        book.remove_order(Side::Sell, 1100, 50, 5).unwrap();
        assert!(book.asks().get(&1100).is_none());
        assert_eq!(book.timestamp, 5);
    }

    #[test]
    fn errors_if_shares_exceed_available() {
        let mut book = OrderBook::new("01/01/2015".to_string(), "XYZ".to_string(), 3);

        book.add_order(Side::Buy, 1000, 100, 0);
        assert!(book.bids().contains_key(&1000));
        assert_eq!(*book.bids().get(&1000).unwrap(), 100);

        let result = book.remove_order(Side::Buy, 1000, 200, 1);
        assert!(result.is_err());
    }
}
