use std::{
    error::Error,
    fs::{create_dir_all, remove_file, File, OpenOptions},
    path::{Path, PathBuf},
};

use csv::WriterBuilder;

use super::{Flush, ALL_STEM};
use crate::{
    message::{NOIIMessage, OrderMessage, TradeMessage},
    orderbook::OrderBookSnapshot,
};

/// The output collections produced by a parse run. Each maps to a top-level
/// subdirectory under the output dir.
#[derive(Clone, Copy)]
pub enum Collection {
    Orders,
    Trades,
    Noii,
    Books,
}

impl Collection {
    fn dir(self) -> &'static str {
        match self {
            Collection::Orders => "orders",
            Collection::Trades => "trades",
            Collection::Noii => "noii",
            Collection::Books => "books",
        }
    }

    /// All collections, for sweeping every output a ticker may have produced.
    const ALL: [Collection; 4] = [
        Collection::Orders,
        Collection::Trades,
        Collection::Noii,
        Collection::Books,
    ];
}

pub struct CSV {
    output_dir: PathBuf,
    /// When set, all messages collapse into a single `_all.csv` per collection
    /// rather than one file per ticker (the `--tickers *` case).
    wildcard: bool,
}

impl CSV {
    pub fn new<P: AsRef<Path>>(output_dir: P, wildcard: bool) -> std::io::Result<Self> {
        let path = output_dir.as_ref().to_path_buf();
        if !path.exists() {
            create_dir_all(&path)?;
        }

        Ok(Self {
            output_dir: path,
            wildcard,
        })
    }

    /// The filename stem for a flush: the message's own ticker, or the combined
    /// `_all` stem under the wildcard. The one place the collapse is decided.
    fn file_stem<'a>(&self, ticker: &'a str) -> &'a str {
        if self.wildcard {
            ALL_STEM
        } else {
            ticker
        }
    }

    /// The collection's date directory: `<dir>/<collection>/<date>`.
    fn date_dir(&self, date: &str, collection: Collection) -> PathBuf {
        self.output_dir.join(collection.dir()).join(date)
    }

    /// The single source of truth for output paths:
    /// `<dir>/<collection>/<date>/<stem>.csv`.
    fn path_to(&self, date: &str, stem: &str, collection: Collection) -> PathBuf {
        self.date_dir(date, collection)
            .join(format!("{}.csv", stem))
    }

    /// Existing `.csv` files in a collection's date directory.
    fn existing_csvs(&self, date: &str, collection: Collection) -> Vec<PathBuf> {
        let dir = self.date_dir(date, collection);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return vec![];
        };
        entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "csv"))
            .collect()
    }

    /// Existing output files a run would conflict with. Wildcard and per-ticker
    /// output are mutually exclusive within a partition: a wildcard run conflicts
    /// with any existing file in the date dir, and a per-ticker run conflicts with
    /// its own `<ticker>.csv` or an existing `_all.csv`.
    pub fn check_collisions(&self, date: &str, tickers: &[String]) -> Vec<PathBuf> {
        let mut collisions = vec![];
        for collection in Collection::ALL {
            if self.wildcard {
                collisions.extend(self.existing_csvs(date, collection));
            } else {
                let all = self.path_to(date, ALL_STEM, collection);
                if all.exists() {
                    collisions.push(all);
                }
                for ticker in tickers {
                    let path = self.path_to(date, ticker, collection);
                    if path.exists() {
                        collisions.push(path);
                    }
                }
            }
        }
        collisions
    }

    /// Delete the given colliding files. Consumes paths produced by
    /// [`check_collisions`]; does not re-derive them.
    pub fn clear_collisions(&self, collisions: &[PathBuf]) -> std::io::Result<()> {
        for path in collisions {
            remove_file(path)?;
        }
        Ok(())
    }

    /// Open (create-or-append) the file for `ticker` in `collection`, creating
    /// the collection/date directory tree as needed. Returns the file and
    /// whether it already existed (so callers can decide whether to write headers).
    fn open_for(
        &self,
        date: &str,
        ticker: &str,
        collection: Collection,
    ) -> std::io::Result<(File, bool)> {
        let filepath = self.path_to(date, self.file_stem(ticker), collection);
        if let Some(parent) = filepath.parent() {
            create_dir_all(parent)?;
        }
        let file_exists = filepath.exists();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)?;
        Ok((file, file_exists))
    }
}

impl Flush for CSV {
    fn flush_order_messages(&self, order_messages: &[OrderMessage]) -> Result<(), Box<dyn Error>> {
        if order_messages.is_empty() {
            return Ok(());
        }

        // Assume same date and ticker across all messages (one ticker per flush).
        let date = order_messages[0].date();
        let ticker = order_messages[0].ticker();

        let (file, file_exists) = self.open_for(date, ticker, Collection::Orders)?;
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
        if snapshots.is_empty() {
            return Ok(());
        }

        // Assume same date and ticker across all snapshots (one ticker per flush).
        let date = snapshots[0].date();
        let ticker = snapshots[0].ticker();

        let (file, file_exists) = self.open_for(date, ticker, Collection::Books)?;
        let mut writer = WriterBuilder::new()
            .has_headers(false) // We'll write headers manually
            .from_writer(file);

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

        Ok(())
    }

    fn flush_trade_messages(&self, trade_messages: &[TradeMessage]) -> Result<(), Box<dyn Error>> {
        if trade_messages.is_empty() {
            return Ok(());
        }

        // Assume same date and ticker across all messages (one ticker per flush).
        let date = trade_messages[0].date();
        let ticker = trade_messages[0].ticker();

        let (file, file_exists) = self.open_for(date, ticker, Collection::Trades)?;
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
        if noii_messages.is_empty() {
            return Ok(());
        }

        // Assume same date and ticker across all messages (one ticker per flush).
        let date = noii_messages[0].date();
        let ticker = noii_messages[0].ticker();

        let (file, file_exists) = self.open_for(date, ticker, Collection::Noii)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn backend(wildcard: bool) -> CSV {
        // Construct directly to avoid touching the filesystem in CSV::new.
        CSV {
            output_dir: PathBuf::from("data"),
            wildcard,
        }
    }

    #[test]
    fn path_uses_ticker_when_not_wildcard() {
        let csv = backend(false);
        assert_eq!(
            csv.path_to("2017-02-27", csv.file_stem("AAPL"), Collection::Orders),
            PathBuf::from("data/orders/2017-02-27/AAPL.csv")
        );
    }

    #[test]
    fn path_collapses_to_all_when_wildcard() {
        let csv = backend(true);
        // Even given a real ticker, the stem collapses to `_all`.
        assert_eq!(
            csv.path_to("2017-02-27", csv.file_stem("AAPL"), Collection::Trades),
            PathBuf::from("data/trades/2017-02-27/_all.csv")
        );
    }

    #[test]
    fn each_collection_maps_to_its_own_dir() {
        let csv = backend(false);
        let p = |c| csv.path_to("d", "T", c);
        assert_eq!(p(Collection::Orders), PathBuf::from("data/orders/d/T.csv"));
        assert_eq!(p(Collection::Trades), PathBuf::from("data/trades/d/T.csv"));
        assert_eq!(p(Collection::Noii), PathBuf::from("data/noii/d/T.csv"));
        assert_eq!(p(Collection::Books), PathBuf::from("data/books/d/T.csv"));
    }

    #[test]
    fn wildcard_path_stem_is_all_not_literal_star() {
        // Under wildcard a flush writes `_all.csv`, never a file named `*`.
        let csv = backend(true);
        let path = csv.path_to("d", csv.file_stem("*"), Collection::Orders);
        assert_eq!(path, PathBuf::from("data/orders/d/_all.csv"));
    }

    #[test]
    fn date_dir_nests_collection_and_date() {
        let csv = backend(false);
        assert_eq!(
            csv.date_dir("2017-02-27", Collection::Books),
            PathBuf::from("data/books/2017-02-27")
        );
    }

    // --- Collision guard (filesystem-backed via tempdir) ---

    use tempfile::TempDir;

    const DATE: &str = "2017-02-27";

    /// A CSV backend rooted at a fresh tempdir. The dir is removed when the
    /// returned guard drops, so the tempdir must outlive the CSV.
    fn temp_backend(wildcard: bool) -> (TempDir, CSV) {
        let tmp = TempDir::new().unwrap();
        let csv = CSV::new(tmp.path(), wildcard).unwrap();
        (tmp, csv)
    }

    /// Create an empty output file for `stem` in `collection` under `date`.
    fn touch(csv: &CSV, date: &str, stem: &str, collection: Collection) -> PathBuf {
        let path = csv.path_to(date, stem, collection);
        create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "").unwrap();
        path
    }

    #[test]
    fn no_collision_when_partition_empty() {
        let (_tmp, csv) = temp_backend(false);
        assert!(csv.check_collisions(DATE, &["AAPL".to_string()]).is_empty());
    }

    #[test]
    fn per_ticker_collides_only_with_named_tickers() {
        let (_tmp, csv) = temp_backend(false);
        let aapl = touch(&csv, DATE, "AAPL", Collection::Orders);
        touch(&csv, DATE, "MSFT", Collection::Orders);

        // Checking only AAPL must not flag MSFT — overwrite is surgical.
        let hits = csv.check_collisions(DATE, &["AAPL".to_string()]);
        assert_eq!(hits, vec![aapl]);
    }

    #[test]
    fn per_ticker_collides_with_existing_all_file() {
        // A per-ticker run into a partition already written by a wildcard run.
        let (_tmp, csv) = temp_backend(false);
        let all = touch(&csv, DATE, ALL_STEM, Collection::Orders);

        let hits = csv.check_collisions(DATE, &["AAPL".to_string()]);
        assert!(
            hits.contains(&all),
            "per-ticker run must conflict with _all.csv"
        );
    }

    #[test]
    fn wildcard_collides_with_existing_per_ticker_files() {
        // The original gap: wildcard run into a partition with per-ticker files
        // but no _all.csv. Must still detect the conflict.
        let (_tmp, csv) = temp_backend(true);
        let aapl = touch(&csv, DATE, "AAPL", Collection::Orders);
        let msft = touch(&csv, DATE, "MSFT", Collection::Orders);

        let hits = csv.check_collisions(DATE, &["*".to_string()]);
        assert!(hits.contains(&aapl) && hits.contains(&msft));
    }

    #[test]
    fn clear_collisions_removes_files() {
        let (_tmp, csv) = temp_backend(false);
        let aapl = touch(&csv, DATE, "AAPL", Collection::Orders);

        let hits = csv.check_collisions(DATE, &["AAPL".to_string()]);
        csv.clear_collisions(&hits).unwrap();
        assert!(!aapl.exists());
        assert!(csv.check_collisions(DATE, &["AAPL".to_string()]).is_empty());
    }
}
