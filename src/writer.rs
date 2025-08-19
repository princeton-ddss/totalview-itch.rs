mod csv;

use std::error::Error;

use crate::message::OrderMessage;
use crate::order_book::OrderBookSnapshot;

pub use csv::CSV;

pub trait Flush {
    fn flush_order_messages(&self, order_messages: &[OrderMessage]) -> Result<(), Box<dyn Error>>;
    fn flush_snapshots(&self, snapshots: &[OrderBookSnapshot]) -> Result<(), Box<dyn Error>>;
}

pub struct Writer<T: Flush> {
    backend: T,
    order_messages: Vec<OrderMessage>,
    snapshots: Vec<OrderBookSnapshot>,
    buffer_size: usize,
}

impl<T: Flush> Writer<T> {
    pub fn new(backend: T, buffer_size: usize) -> Self {
        Self {
            backend,
            order_messages: vec![],
            snapshots: vec![],
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

    pub fn write_snapshot(
        &mut self,
        snapshot: OrderBookSnapshot,
    ) -> Result<(), Box<dyn Error>> {
        self.snapshots.push(snapshot);

        if self.snapshots.len() >= self.buffer_size {
            self.backend.flush_snapshots(&self.snapshots)?;
            self.snapshots.clear();
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
    }
}
