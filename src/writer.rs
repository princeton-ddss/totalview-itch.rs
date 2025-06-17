mod csv;

use std::error::Error;

use crate::message::OrderMessage;

pub use csv::CSV;

pub trait Flush {
    fn flush_order_messages(&self, order_messages: &[OrderMessage]) -> Result<(), Box<dyn Error>>;
}

pub struct Writer<const N: usize, T: Flush> {
    backend: T,
    order_messages: Vec<OrderMessage>,
}

impl<const N: usize, T: Flush> Writer<N, T> {
    pub fn new(backend: T) -> Self {
        Self {
            backend,
            order_messages: vec![],
        }
    }

    pub fn write_order_message(
        &mut self,
        order_message: OrderMessage,
    ) -> Result<(), Box<dyn Error>> {
        self.order_messages.push(order_message);

        if self.order_messages.len() >= N {
            self.backend.flush_order_messages(&self.order_messages)?;
            self.order_messages.clear();
        }

        Ok(())
    }
}

impl<const N: usize, T: Flush> Drop for Writer<N, T> {
    fn drop(&mut self) {
        if !self.order_messages.is_empty() {
            match self.backend.flush_order_messages(&self.order_messages) {
                Err(e) => eprintln!("Failed to flush residual order messages: {}", e),
                Ok(_) => self.order_messages.clear(),
            };
        }
    }
}
