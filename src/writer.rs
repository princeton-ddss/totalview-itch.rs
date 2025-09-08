mod csv;

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
    order_messages: Vec<OrderMessage>,
    snapshots: Vec<OrderBookSnapshot>,
    trade_messages: Vec<TradeMessage>,
    noii_messages: Vec<NOIIMessage>,
    buffer_size: usize,
}

impl<T: Flush> Writer<T> {
    pub fn new(backend: T, buffer_size: usize) -> Self {
        Self {
            backend,
            order_messages: vec![],
            snapshots: vec![],
            trade_messages: vec![],
            noii_messages: vec![],
            buffer_size,
        }
    }

    pub fn write_order_message(
        &mut self,
        order_message: OrderMessage,
    ) -> Result<(), Box<dyn Error>> {
        self.order_messages.push(order_message);

        if self.order_messages.len() >= self.buffer_size {
            self.backend.flush_order_messages(&self.order_messages)?;
            self.order_messages.clear();
        }

        Ok(())
    }

    pub fn write_snapshot(&mut self, snapshot: OrderBookSnapshot) -> Result<(), Box<dyn Error>> {
        self.snapshots.push(snapshot);

        if self.snapshots.len() >= self.buffer_size {
            self.backend.flush_snapshots(&self.snapshots)?;
            self.snapshots.clear();
        }

        Ok(())
    }

    pub fn write_trade_message(
        &mut self,
        trade_message: TradeMessage,
    ) -> Result<(), Box<dyn Error>> {
        self.trade_messages.push(trade_message);

        if self.trade_messages.len() >= self.buffer_size {
            self.backend.flush_trade_messages(&self.trade_messages)?;
            self.trade_messages.clear();
        }

        Ok(())
    }

    pub fn write_noii_message(&mut self, noii_message: NOIIMessage) -> Result<(), Box<dyn Error>> {
        self.noii_messages.push(noii_message);

        if self.noii_messages.len() >= self.buffer_size {
            self.backend.flush_noii_messages(&self.noii_messages)?;
            self.noii_messages.clear();
        }

        Ok(())
    }
}

impl<T: Flush> Drop for Writer<T> {
    fn drop(&mut self) {
        if !self.order_messages.is_empty() {
            match self.backend.flush_order_messages(&self.order_messages) {
                Err(e) => eprintln!("Failed to flush residual order messages: {}", e),
                Ok(_) => self.order_messages.clear(),
            };
        }

        if !self.snapshots.is_empty() {
            match self.backend.flush_snapshots(&self.snapshots) {
                Err(e) => eprintln!("Failed to flush residual snapshots: {}", e),
                Ok(_) => self.snapshots.clear(),
            };
        }

        if !self.trade_messages.is_empty() {
            match self.backend.flush_trade_messages(&self.trade_messages) {
                Err(e) => eprintln!("Failed to flush residual trade messages: {}", e),
                Ok(_) => self.trade_messages.clear(),
            };
        }

        if !self.noii_messages.is_empty() {
            match self.backend.flush_noii_messages(&self.noii_messages) {
                Err(e) => eprintln!("Failed to flush residual noii messages: {}", e),
                Ok(_) => self.noii_messages.clear(),
            };
        }
    }
}
