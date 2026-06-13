mod csv;

use std::collections::HashMap;
use std::error::Error;

pub use csv::CSV;

use crate::{
    message::{NOIIMessage, OrderMessage, TradeMessage},
    orderbook::OrderBookSnapshot,
};

pub trait Flush {
    fn flush_order_messages(&self, order_messages: &[OrderMessage]) -> Result<(), Box<dyn Error>>;
    fn flush_snapshots(&self, snapshots: &[OrderBookSnapshot]) -> Result<(), Box<dyn Error>>;
    fn flush_trade_messages(&self, trade_messages: &[TradeMessage]) -> Result<(), Box<dyn Error>>;
    fn flush_noii_messages(&self, noii_messages: &[NOIIMessage]) -> Result<(), Box<dyn Error>>;
}

pub struct Writer<T: Flush> {
    backend: T,
    order_messages: HashMap<String, Vec<OrderMessage>>,
    order_buffered: usize,
    snapshots: HashMap<String, Vec<OrderBookSnapshot>>,
    snapshots_buffered: usize,
    trade_messages: HashMap<String, Vec<TradeMessage>>,
    trade_buffered: usize,
    noii_messages: HashMap<String, Vec<NOIIMessage>>,
    noii_buffered: usize,
    buffer_size: usize,
    max_buffered: usize,
    min_flush_size: usize,
    /// When set, all messages bucket under a single `_all` key instead of one
    /// vec per ticker (the `--tickers *` case). Keeps the wildcard run from
    /// fragmenting memory across thousands of per-ticker vecs.
    wildcard: bool,
}

/// Buffer key and filename stem for the combined output under the `*` wildcard.
pub(crate) const ALL_STEM: &str = "_all";

impl<T: Flush> Writer<T> {
    pub fn new(
        backend: T,
        buffer_size: usize,
        max_buffered: usize,
        min_flush_size: usize,
        wildcard: bool,
    ) -> Self {
        assert!(
            min_flush_size < buffer_size,
            "min_flush_size ({}) should be < buffer_size ({})",
            min_flush_size,
            buffer_size
        );
        assert!(
            buffer_size <= max_buffered,
            "buffer_size ({}) should be <= max_buffered ({})",
            buffer_size,
            max_buffered
        );

        let order_messages = HashMap::new();
        let snapshots = HashMap::new();
        let trade_messages = HashMap::new();
        let noii_messages = HashMap::new();

        Self {
            backend,
            order_messages,
            order_buffered: 0,
            snapshots,
            snapshots_buffered: 0,
            trade_messages,
            trade_buffered: 0,
            noii_messages,
            noii_buffered: 0,
            buffer_size,
            max_buffered,
            min_flush_size,
            wildcard,
        }
    }

    /// The buffer key for a message's ticker: `_all` under the wildcard,
    /// otherwise the ticker itself.
    fn bucket_key(&self, ticker: &str) -> String {
        if self.wildcard {
            ALL_STEM.to_string()
        } else {
            ticker.to_string()
        }
    }

    pub fn write_order_message(
        &mut self,
        order_message: OrderMessage,
    ) -> Result<(), Box<dyn Error>> {
        let key = self.bucket_key(order_message.ticker());
        let messages = self.order_messages.entry(key).or_default();

        // Push order message
        messages.push(order_message);
        self.order_buffered += 1;

        // Clean up messages
        if messages.len() >= self.buffer_size {
            // This ticker has too many messages => flush it
            let n = messages.len();
            self.backend.flush_order_messages(messages)?;
            self.order_buffered -= n;
            messages.clear();
        }

        if self.order_buffered > self.max_buffered {
            // We have too many messages overall => let's clean up a bit
            self.prune_order_messages(self.min_flush_size)?;
        }

        if self.order_buffered > self.max_buffered {
            // We have too many messages overall => let's clean up a bit
            self.prune_order_messages(0)?;
        }

        Ok(())
    }

    fn prune_order_messages(&mut self, threshold: usize) -> Result<(), Box<dyn Error>> {
        for (_, vec) in self.order_messages.iter_mut() {
            if vec.len() >= threshold {
                let n = vec.len();
                self.backend.flush_order_messages(vec)?;
                self.order_buffered -= n;
                vec.clear();
            }
        }

        Ok(())
    }

    pub fn write_snapshot(&mut self, snapshot: OrderBookSnapshot) -> Result<(), Box<dyn Error>> {
        let key = self.bucket_key(snapshot.ticker());
        let snapshots = self.snapshots.entry(key).or_default();

        // Push snapshot
        snapshots.push(snapshot);
        self.snapshots_buffered += 1;

        // Clean up snapshots
        if snapshots.len() >= self.buffer_size {
            // This ticker has too many snapshots => flush it
            let n = snapshots.len();
            self.backend.flush_snapshots(snapshots)?;
            self.snapshots_buffered -= n;
            snapshots.clear();
        }

        if self.snapshots_buffered > self.max_buffered {
            // We have too many snapshots overall => let's clean up a bit
            self.prune_snapshots(self.min_flush_size)?;
        }

        if self.snapshots_buffered > self.max_buffered {
            // We have too many snapshots overall => let's clean up a bit
            self.prune_snapshots(0)?;
        }

        Ok(())
    }

    fn prune_snapshots(&mut self, threshold: usize) -> Result<(), Box<dyn Error>> {
        for (_, vec) in self.snapshots.iter_mut() {
            if vec.len() >= threshold {
                let n = vec.len();
                self.backend.flush_snapshots(vec)?;
                self.snapshots_buffered -= n;
                vec.clear();
            }
        }

        Ok(())
    }

    pub fn write_trade_message(
        &mut self,
        trade_message: TradeMessage,
    ) -> Result<(), Box<dyn Error>> {
        let key = self.bucket_key(trade_message.ticker());
        let messages = self.trade_messages.entry(key).or_default();

        // Push order message
        messages.push(trade_message);
        self.trade_buffered += 1;

        // Clean up messages
        if messages.len() >= self.buffer_size {
            // This ticker has too many messages => flush it
            let n = messages.len();
            self.backend.flush_trade_messages(messages)?;
            self.trade_buffered -= n;
            messages.clear();
        }

        if self.trade_buffered > self.max_buffered {
            // We have too many messages overall => let's clean up a bit
            self.prune_trade_messages(self.min_flush_size)?;
        }

        if self.trade_buffered > self.max_buffered {
            // We have too many messages overall => let's clean up a bit
            self.prune_trade_messages(0)?;
        }

        Ok(())
    }

    fn prune_trade_messages(&mut self, threshold: usize) -> Result<(), Box<dyn Error>> {
        for (_, vec) in self.trade_messages.iter_mut() {
            if vec.len() >= threshold {
                let n = vec.len();
                self.backend.flush_trade_messages(vec)?;
                self.trade_buffered -= n;
                vec.clear();
            }
        }

        Ok(())
    }

    pub fn write_noii_message(&mut self, noii_message: NOIIMessage) -> Result<(), Box<dyn Error>> {
        let key = self.bucket_key(noii_message.ticker());
        let messages = self.noii_messages.entry(key).or_default();

        // Push NOII message
        messages.push(noii_message);
        self.noii_buffered += 1;

        // Clean up messages
        if messages.len() >= self.buffer_size {
            // This ticker has too many messages => flush it
            let n = messages.len();
            self.backend.flush_noii_messages(messages)?;
            self.noii_buffered -= n;
            messages.clear();
        }

        if self.noii_buffered > self.max_buffered {
            // We have too many messages overall => let's clean up a bit
            self.prune_noii_messages(self.min_flush_size)?;
        }

        if self.noii_buffered > self.max_buffered {
            // We have too many messages overall => let's clean up a bit
            self.prune_noii_messages(0)?;
        }

        Ok(())
    }

    fn prune_noii_messages(&mut self, threshold: usize) -> Result<(), Box<dyn Error>> {
        for (_, vec) in self.noii_messages.iter_mut() {
            if vec.len() >= threshold {
                let n = vec.len();
                self.backend.flush_noii_messages(vec)?;
                self.noii_buffered -= n;
                vec.clear();
            }
        }

        Ok(())
    }
}

impl<T: Flush> Drop for Writer<T> {
    fn drop(&mut self) {
        if self.order_buffered > 0 {
            if let Err(e) = self.prune_order_messages(0) {
                eprintln!("Failed to flush residual order messages: {}", e);
            };
        }

        if self.snapshots_buffered > 0 {
            if let Err(e) = self.prune_snapshots(0) {
                eprintln!("Failed to flush residual snapshots: {}", e);
            };
        }

        if self.trade_buffered > 0 {
            if let Err(e) = self.prune_trade_messages(0) {
                eprintln!("Failed to flush residual trade messages: {}", e);
            };
        }

        if self.noii_buffered > 0 {
            if let Err(e) = self.prune_noii_messages(0) {
                eprintln!("Failed to flush residual NOII messages: {}", e);
            };
        }
    }
}
