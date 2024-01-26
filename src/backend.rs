use std::fs::File;
use csv;
use serde::Serialize;

use crate::Message;

pub struct Backend {
    writer: csv::Writer<File>,
    pos: usize,
    size: usize,
}

impl Backend {
    
    pub fn new(writer: csv::Writer<File>, size: usize) -> Self {
        Self { writer, pos: 0, size }
    }

    pub fn write<T: Serialize + Message>(&mut self, item: T) -> std::io::Result<()> {
        self.writer.serialize(item)?;
        self.pos += 1;

        if self.pos == self.size {
            let _ = self.flush();
        }

        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        // println!("Flushing data. Records to write: {}", self.pos);
        self.writer.flush()?;
        self.pos = 0;

        Ok(())
    }
}

// struct Writer<T: Message, W: Write> {
//     buffer: BufWriter<W>,
//     pos: usize,
//     max_size: usize,
// }

// impl<T: Message,W: Write> Writer<T, W> {

//     pub fn write(&mut self, item: T) -> std::io::Result<()> {
//         self.buffer.write(to_string(item).as_bytes())?;
//         self.pos += 1;
//         if self.pos == self.max_size {
//             self.buffer.flush()?;
//             self.pos = 0;
//         }

//         Ok(())
//     }



//     pub fn flush(&mut self) {
//         // Write whatever is in the buffer to file
//         let mut options = OpenOptions::new();
//         let mut file = options.append(true).create(true).open(&self.dir);
        
//     }
// }

// pub fn create_csv() -> std::io::Result<()> {
//     let mut file = File::create("./data/test.csv").unwrap();
//     let line = String::from("date,time,type\n");
//     file.write_all(line.as_bytes()).unwrap();
//     Ok(())
// }

// pub fn append_csv() -> std::io::Result<()> {
//     let mut options = OpenOptions::new();
//     let mut file = options.append(true).open("./data/test.csv").unwrap();
//     let line = String::from("date,time,type\n");
//     file.write_all(line.as_bytes()).unwrap();
//     Ok(())
// }



// pub fn read_csv() {

// }