use std::error::Error;
use std::fs::{create_dir, create_dir_all, OpenOptions};
use std::path::{Path, PathBuf};

use csv::WriterBuilder;

use crate::message::{OrderMessage, TradeMessage, NOIIMessage};
use crate::orderbook::OrderBookSnapshot;

use super::Flush;

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
        let dirpath = self.output_dir.join("order_messages");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
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
        let dirpath = self.output_dir.join("snapshots");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
        }

        // Group snapshots by ticker
        let mut ticker_snapshots: std::collections::HashMap<String, Vec<&OrderBookSnapshot>> = 
            std::collections::HashMap::new();
        
        for snapshot in snapshots {
            ticker_snapshots
                .entry(snapshot.ticker.clone())
                .or_insert_with(Vec::new)
                .push(snapshot);
        }

        // Write each ticker's snapshots to its own file
        for (ticker, ticker_snaps) in ticker_snapshots {
            let filename = format!("{}_snapshots.csv", ticker);
            let filepath = dirpath.join(filename);
            let file_exists = filepath.exists();

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(filepath)?;

            let mut writer = WriterBuilder::new()
                .has_headers(false) // We'll write headers manually
                .from_writer(file);

            // Write headers if file is new
            if !file_exists && !ticker_snaps.is_empty() {
                let levels_count = ticker_snaps[0].levels.len() / 4; // levels per side
                let mut headers = vec!["ticker".to_string(), "timestamp".to_string()];
                
                // Add bid headers
                for i in 1..=levels_count {
                    headers.push(format!("bid_price_{}", i));
                    headers.push(format!("bid_size_{}", i));
                }
                
                // Add ask headers  
                for i in 1..=levels_count {
                    headers.push(format!("ask_price_{}", i));
                    headers.push(format!("ask_size_{}", i));
                }
                
                writer.write_record(&headers)?;
            }

            // Write data rows
            for snapshot in ticker_snaps {
                let mut record = vec![snapshot.ticker.clone(), snapshot.timestamp.to_string()];
                for level in &snapshot.levels {
                    record.push(level.to_string());
                }
                writer.write_record(&record)?;
            }

            writer.flush()?;
        }

        Ok(())
    }

    fn flush_trade_messages(&self, trade_messages: &[TradeMessage]) -> Result<(), Box<dyn Error>> {
        let dirpath = self.output_dir.join("trade_messages");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
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
        let dirpath = self.output_dir.join("noii_messages");
        if !dirpath.exists() {
            create_dir(&dirpath)?;
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
