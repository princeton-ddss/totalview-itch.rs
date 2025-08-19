use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use lobsters::{message::IntoOrderMessage, BufFile, Buffer, Message, Reader, Version, Writer, CSV};
use std::fs;
use std::io::{ErrorKind, Seek};

// TODO: Create CLI argparser object (clap)
// TODO: Use logging instead of println!
// TODO: Add progress bar? (Can be based on bytes read / length of file)
// TODO: Derive date from file name
// TODO: Handle end of file logic

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
        default_value_t = 1028,
        help = "The size of internal buffer used for reading data."
    )]
    capacity: usize,
}

fn main() {
    let args = Cli::parse();

    let filesize = fs::metadata(&args.path).unwrap().len();

    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            ).unwrap()
            .progress_chars("#>-"),
    );

    // let mut buffer = BufFile::with_capacity(args.capacity, &args.path).unwrap();
    let mut buffer = Buffer::new(&args.path).unwrap();

    let tickers = args.tickers.split(',').map(|s| s.to_string()).collect();

    let mut reader = Reader::new(Version::V50, tickers);

    let backend = CSV::new("data").unwrap();
    let mut writer = Writer::<10, CSV>::new(backend);

    let date = String::from("2017-02-27"); // To be derived from the file name

    let mut messages_read = 0;

    // for _ in 0..100 {
    loop {
        let current_pos = buffer.stream_position().unwrap();
        pb.set_position(current_pos);

        match reader.extract_message(&mut buffer) {
            Ok(msg) => {
                messages_read += 1;
                pb.set_message(format!("Messages: {}", messages_read));

                match msg {
                    Message::AddOrder(data) => {
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                    }
                    Message::CancelOrder(data) => {
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                    }
                    Message::DeleteOrder(data) => {
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
                    }
                    Message::ExecuteOrder(data) => {
                        let order_message = data.into_order_message(date.clone());
                        writer.write_order_message(order_message).unwrap();
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
                    return; // Err(e); // Actual error
                }
            }
        }
    }
    pb.finish_with_message(format!("✅ Processed {} messages", messages_read));
}
