use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use lobsters::{message::IntoOrderMessage, Buffer, Message, Reader, Version, Writer, CSV};
use std::fs;
use std::io::{ErrorKind, Seek};
use std::path::Path;

// TODO: Use logging instead of println!
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
    // Parse args and environment variables
    let args = Cli::parse();
    let tickers = args.tickers.split(',').map(|s| s.to_string()).collect();
    let (date, version) = parse_filename(&args.path).expect(
        "The filename should match the format 'SMMDDYY-vNN' where 'NN' is one of '41' or '50'.",
    );

    // Set up reader and writer
    let mut buffer = Buffer::new(&args.path).unwrap();
    let mut reader = Reader::new(version, tickers);
    let backend = CSV::new("data").unwrap();
    let mut writer = Writer::<10, CSV>::new(backend);

    // Set up progress bar
    let mut messages_read = 0;
    let filesize = fs::metadata(&args.path).unwrap().len();
    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {msg}, {eta})",
            ).unwrap()
            .progress_chars("#>-"),
    );

    // Begin main loop...
    loop {
        let current_pos = buffer.stream_position().unwrap();
        pb.set_position(current_pos);

        match reader.extract_message(&mut buffer) {
            Ok(msg) => {
                messages_read += 1;
                pb.set_message(format!("{} messages", messages_read));

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
