mod csv;

use std::error::Error;

use crate::message::OrderMessage;

pub use csv::CSV;

pub trait Flush {
    fn flush_order_messages(
        &self,
        order_messages: &mut Vec<OrderMessage>,
    ) -> Result<(), Box<dyn Error>>;
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
            self.backend.flush_order_messages(&mut self.order_messages)
        } else {
            Ok(())
        }
    }
}
