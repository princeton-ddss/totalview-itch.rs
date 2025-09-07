use std::{
    error::Error,
    fs::{create_dir, create_dir_all, OpenOptions},
    path::{Path, PathBuf},
};

use csv::WriterBuilder;

use super::Flush;
use crate::{
    message::{NOIIMessage, OrderMessage, TradeMessage},
    orderbook::OrderBookSnapshot,
};

pub struct CSV {
    output_dir: PathBuf,
}

impl CSV {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> std::io::Result<Self> {
        let path = output_dir.as_ref().to_path_buf();
        if !path.exists() {
            create_dir_all(&path)?;
        }

        Ok(Self { output_dir: path })
    }
}

impl Flush for CSV {
    fn flush_order_messages(&self, order_messages: &[OrderMessage]) -> Result<(), Box<dyn Error>> {
        let dirpath = self.output_dir.join("orders");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
        }

        if order_messages.is_empty() {
            return Ok(());
        }

        let date = order_messages[0].date().clone(); // Assume same date across all messages
        let filename = format!("{}.csv", date);
        let filepath = dirpath.join(filename);
        let file_exists = filepath.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)?;

        let mut writer = WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(file);

        for message in order_messages {
            writer.serialize(message)?;
        }

        writer.flush()?;

        Ok(())
    }

    fn flush_snapshots(&self, snapshots: &[OrderBookSnapshot]) -> Result<(), Box<dyn Error>> {
        let dirpath = self.output_dir.join("books");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
        }

        let date = snapshots[0].date.clone(); // Assume same date across all messages
        let filename = format!("{}.csv", date);
        let filepath = dirpath.join(filename);
        let file_exists = filepath.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)?;

        let mut writer = WriterBuilder::new()
            .has_headers(false) // We'll write headers manually
            .from_writer(file);

        if !snapshots.is_empty() {
            // Write headers if file is new
            if !file_exists {
                let levels_count = snapshots[0].data.len() / 4; // levels per side
                let mut headers = vec!["ticker".to_string(), "timestamp".to_string()];

                for i in 1..=levels_count {
                    headers.push(format!("bid_price_{}", i));
                    headers.push(format!("bid_size_{}", i));
                }

                for i in 1..=levels_count {
                    headers.push(format!("ask_price_{}", i));
                    headers.push(format!("ask_size_{}", i));
                }

                writer.write_record(&headers)?;
            }

            // Write data rows
            for snapshot in snapshots {
                let mut record = vec![snapshot.ticker.clone(), snapshot.timestamp.to_string()];
                for val in &snapshot.data {
                    record.push(val.to_string());
                }
                writer.write_record(&record)?;
            }

            writer.flush()?;
        }

        Ok(())
    }

    fn flush_trade_messages(&self, trade_messages: &[TradeMessage]) -> Result<(), Box<dyn Error>> {
        let dirpath = self.output_dir.join("trades");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
        }

        if trade_messages.is_empty() {
            return Ok(());
        }

        let date = trade_messages[0].date().clone(); // Assume same date across all messages
        let filename = format!("{}.csv", date);
        let filepath = dirpath.join(filename);
        let file_exists = filepath.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)?;

        let mut writer = WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(file);

        for message in trade_messages {
            writer.serialize(message)?;
        }

        writer.flush()?;

        Ok(())
    }

    fn flush_noii_messages(&self, noii_messages: &[NOIIMessage]) -> Result<(), Box<dyn Error>> {
        let dirpath = self.output_dir.join("noii");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
        }

        if noii_messages.is_empty() {
            return Ok(());
        }

        let date = noii_messages[0].date().clone(); // Assume same date across all messages
        let filename = format!("{}.csv", date);
        let filepath = dirpath.join(filename);
        let file_exists = filepath.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)?;

        let mut writer = WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(file);

        for message in noii_messages {
            writer.serialize(message)?;
        }

        writer.flush()?;

        Ok(())
    }
}
