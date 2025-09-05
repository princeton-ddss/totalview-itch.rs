use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use lobsters::{
    message::{IntoNOIIMessage, IntoOrderMessage, IntoTradeMessage},
    Buffer, Message, OrderBook, Reader, Version, Writer, CSV,
};
use std::collections::HashSet;
use std::fs;
use std::io::{ErrorKind, Seek};
use std::path::Path;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// TODO: Print error to std:err
// TODO: handle panics

struct PerformanceMetrics {
    file_size: u64,
    duration: DurationMetrics,
    messages: MessageMetrics,
    // memory: MemoryMetrics,
}

impl PerformanceMetrics {
    fn new(file_size: u64) -> Self {
        Self {
            file_size,
            duration: DurationMetrics::new(),
            messages: MessageMetrics::new(),
            // memory: MemoryMetrics::new(),
        }
    }

    fn summarize(&self) {
        println!("📊 Performance Report");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("File size:      {} MB", self.file_size / 1_000_000);
        println!("Total time:     {:.2}s", self.duration.total.as_secs_f64());
        println!(
            "Throughput:     {:.1} MB/s",
            (self.file_size as f64) / 1_000_000.0 / self.duration.total.as_secs_f64()
        );

        println!("\n🔍 Time Breakdown:");
        let total_ms = self.duration.total.as_millis() as f64;
        println!(
            "  Parsing:      {:.1}ms ({:.1}%)",
            self.duration.parsing.as_millis(),
            self.duration.parsing.as_millis() as f64 / total_ms * 100.0
        );
        println!(
            "  Order books:  {:.1}ms ({:.1}%)",
            self.duration.orderbook.as_millis(),
            self.duration.orderbook.as_millis() as f64 / total_ms * 100.0
        );
        println!(
            "  Serialization:  {:.1}ms ({:.1}%)",
            self.duration.serialization.as_millis(),
            self.duration.serialization.as_millis() as f64 / total_ms * 100.0
        );
    }
}

struct DurationMetrics {
    total: Duration,
    parsing: Duration,
    serialization: Duration,
    orderbook: Duration,
}

impl DurationMetrics {
    fn new() -> Self {
        Self {
            total: Duration::new(0, 0),
            parsing: Duration::new(0, 0),
            serialization: Duration::new(0, 0),
            orderbook: Duration::new(0, 0),
        }
    }
}

struct MessageMetrics {
    total: u64,
    orders: u64,
    // system: u64,
    trades: u64,
    noii: u64,
}

impl MessageMetrics {
    fn new() -> Self {
        Self {
            total: 0,
            orders: 0,
            // system: 0,
            trades: 0,
            noii: 0,
        }
    }
}

// struct MemoryMetrics {
//     max: u64,
//     min: u64,
// }

// impl MemoryMetrics {
//     fn new() -> Self {
//         Self { max: 0, min: 0 }
//     }
// }

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,

    #[arg(
        short,
        long,
        default_value_t = String::from("*"),
        help = "A comma-delimited list of tickers to read read, or '*' to read all messages.")]
    tickers: String, // a comma- or space-delimited list

    #[arg(
        short,
        long,
        default_value_t = 3,
        help = "The number of order book levels to track."
    )]
    depth: usize,

    #[arg(
        short,
        long,
        default_value_t = 1028,
        help = "The size of internal buffer used for writing data."
    )]
    capacity: usize,
}

fn parse_filename<P: AsRef<Path>>(path: P) -> Option<(String, Version)> {
    let filename = path.as_ref().file_stem()?.to_str()?;

    // Expected format: SMMDDYY-vXX (e.g., S022717-v50)
    if filename.len() == 11 && filename.chars().next()? == 'S' {
        let parts: Vec<&str> = filename.split('-').collect();
        if parts.len() == 2 {
            let date_part = &parts[0][1..]; // Skip 'S', take MMDDYY
            let version_part = parts[1];

            if date_part.len() == 6 && date_part.chars().all(|c| c.is_ascii_digit()) {
                let mm = &date_part[0..2];
                let dd = &date_part[2..4];
                let yy = &date_part[4..6];
                let yyyy = format!("20{}", yy);
                let date = format!("{}-{}-{}", yyyy, mm, dd);

                if version_part.starts_with('v') && version_part.len() == 3 {
                    let version = match &version_part[1..] {
                        "41" => Version::V41,
                        "50" => Version::V50,
                        _ => return None,
                    };

                    return Some((date, version));
                }
            }
        }
    }

    None
}

fn main() {
    //  Start timer
    let start = Instant::now();

    // Parse args and environment variables
    let args = Cli::parse();
    let tickers: HashSet<String> = args.tickers.split(',').map(|s| s.to_string()).collect();
    let (date, version) = parse_filename(&args.path).expect(
        "The filename should match the format 'SMMDDYY-vNN' where 'NN' is one of '41' or '50'.",
    );

    // Set up reader and writer
    let mut buffer = Buffer::new(&args.path).unwrap();
    let mut reader = Reader::new(version, tickers.clone());
    let backend = CSV::new("data").unwrap();
    let mut writer = Writer::new(backend, args.capacity);

    // Set up progress bar
    let filesize = fs::metadata(&args.path).unwrap().len();
    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {msg}, {eta})",
            ).unwrap()
            .progress_chars("#>-"),
    );

    // Set up metrics
    let mut metrics = PerformanceMetrics::new(filesize);

    // Create order books for each ticker
    let mut order_books: HashMap<String, OrderBook> = HashMap::new();
    for ticker in &tickers {
        if ticker != "*" {
            // Skip wildcard
            order_books.insert(ticker.clone(), OrderBook::new(ticker.clone()));
        }
    }

    // Begin main loop...
    loop {
        let current_pos = buffer.stream_position().unwrap();
        pb.set_position(current_pos);

        let parse_start = Instant::now();
        match reader.extract_message(&mut buffer) {
            Ok(msg) => {
                metrics.messages.total += 1;
                metrics.duration.parsing += parse_start.elapsed();
                pb.set_message(format!("{} messages", &metrics.messages.total));

                match msg {
                    Message::AddOrder(data) => {
                        metrics.messages.orders += 1;
                        // Update order book
                        if let Some(order_book) = order_books.get_mut(data.ticker()) {
                            let order_book_start = Instant::now();
                            order_book.add_order(*data.side(), *data.price(), *data.shares());
                            metrics.duration.orderbook += order_book_start.elapsed();

                            // Create and write snapshot
                            let write_start = Instant::now();
                            let snapshot = order_book.snapshot(*data.nanoseconds(), args.depth); // Top 10 levels
                            writer.write_snapshot(snapshot).unwrap();
                            metrics.duration.orderbook += write_start.elapsed();
                        }

                        let write_start = Instant::now();
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::CancelOrder(data) => {
                        metrics.messages.orders += 1;
                        // Update order book
                        if let Some(order_book) = order_books.get_mut(data.ticker()) {
                            let order_book_start = Instant::now();
                            if let Err(e) =
                                order_book.remove_order(*data.side(), *data.price(), *data.shares())
                            {
                                metrics.duration.orderbook += order_book_start.elapsed();
                                eprintln!("Warning: Failed to cancel order: {}", e);
                            } else {
                                metrics.duration.orderbook += order_book_start.elapsed();
                                // Create and write snapshot only if update succeeded
                                let write_start = Instant::now();
                                let snapshot = order_book.snapshot(*data.nanoseconds(), args.depth);
                                writer.write_snapshot(snapshot).unwrap();
                                metrics.duration.serialization += write_start.elapsed();
                            }
                        }

                        let write_start = Instant::now();
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::DeleteOrder(data) => {
                        metrics.messages.orders += 1;
                        // Update order book
                        if let Some(order_book) = order_books.get_mut(data.ticker()) {
                            let order_book_start = Instant::now();
                            if let Err(e) =
                                order_book.remove_order(*data.side(), *data.price(), *data.shares())
                            {
                                metrics.duration.orderbook += order_book_start.elapsed();
                                eprintln!("Warning: Failed to delete order: {}", e);
                            } else {
                                metrics.duration.orderbook += order_book_start.elapsed();
                                // Create and write snapshot only if update succeeded
                                let write_start = Instant::now();
                                let snapshot = order_book.snapshot(*data.nanoseconds(), args.depth);
                                writer.write_snapshot(snapshot).unwrap();
                                metrics.duration.serialization += write_start.elapsed();
                            }
                        }

                        let write_start = Instant::now();
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::ExecuteOrder(data) => {
                        metrics.messages.orders += 1;
                        // Update order book
                        if let Some(order_book) = order_books.get_mut(data.ticker()) {
                            let order_book_start = Instant::now();
                            if let Err(e) = order_book.execute_order(
                                *data.side(),
                                *data.price(),
                                *data.shares(),
                            ) {
                                metrics.duration.orderbook += order_book_start.elapsed();
                                eprintln!("Warning: Failed to execute order: {}", e);
                            } else {
                                metrics.duration.orderbook += order_book_start.elapsed();
                                // Create and write snapshot only if update succeeded
                                let write_start = Instant::now();
                                let snapshot = order_book.snapshot(*data.nanoseconds(), args.depth);
                                writer.write_snapshot(snapshot).unwrap();
                                metrics.duration.serialization += write_start.elapsed();
                            }
                        }

                        let write_start = Instant::now();
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::Trade(data) => {
                        metrics.messages.trades += 1;
                        let write_start = Instant::now();
                        let trade_message = data.into_trade_message(date.clone());
                        writer.write_trade_message(trade_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::CrossTrade(data) => {
                        metrics.messages.trades += 1;
                        let write_start = Instant::now();
                        let trade_message = data.into_trade_message(date.clone());
                        writer.write_trade_message(trade_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::BrokenTrade(data) => {
                        metrics.messages.trades += 1;
                        let write_start = Instant::now();
                        let trade_message = data.into_trade_message(date.clone());
                        writer.write_trade_message(trade_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    Message::NetOrderImbalanceIndicator(data) => {
                        metrics.messages.noii += 1;
                        let write_start = Instant::now();
                        let noii_message = data.into_noii_message(date.clone());
                        writer.write_noii_message(noii_message).unwrap();
                        metrics.duration.serialization += write_start.elapsed();
                    }
                    _ => {}
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::InvalidData
                    && e.to_string().contains("File stream is complete")
                {
                    break; // End of file reached
                } else {
                    eprintln!("An error occurred: {}.", e);
                    return; // Err(e); // Actual error
                }
            }
        }
    }

    metrics.duration.total += start.elapsed();
    pb.finish_with_message(format!("✅ Processed {} messages", &metrics.messages.total));
    metrics.summarize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_properly_formatted_filename() {
        let path = std::path::PathBuf::from("data/S022717-v50.txt");
        let (date, version) = parse_filename(path).unwrap();
        assert_eq!(date, "2017-02-27");
        assert_eq!(version, Version::V50);
    }

    #[test]
    fn return_none_if_version_is_invalid() {
        let path = std::path::PathBuf::from("data/S022717-v60.txt");
        let result = parse_filename(path);
        assert!(result.is_none());
    }

    #[test]
    fn return_none_if_date_is_malformed() {
        let path = std::path::PathBuf::from("data/S02272017-v50.txt");
        let result = parse_filename(path);
        assert!(result.is_none());
    }

    #[test]
    fn return_none_if_filename_is_malformed() {
        let path = std::path::PathBuf::from("data/S02272017v50.txt");
        let result = parse_filename(path);
        assert!(result.is_none());
    }
}
