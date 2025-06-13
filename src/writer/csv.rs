use std::error::Error;
use std::fs::{create_dir, create_dir_all, OpenOptions};
use std::path::{Path, PathBuf};

use csv::WriterBuilder;

use crate::message::OrderMessage;

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
}
