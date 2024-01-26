use std::fs::read;
use std::io::{Cursor, Read};
use byteorder::{NetworkEndian, ReadBytesExt};
use std::time::Instant;
use csv;

// [... 0x00 0x23   0x56      0x00 0x00      0x00 0x00 ...]
//      |- size     |- type   |- message
//                            |- time        |- ticker
// 
// struct TimeMessage
//     // length and type are fixed for a given message type
//     pos: usize
// end
// 
// impl TimeMessage {
//     fn get_time(&self, parser: &Parser) -> u32 {
//         // move parser.cursor position to self.pos + time_offset (e.g., 0)
//         // cursor.read_u32::<NetworkEndian>().unwrap()
//     }
// }
// 
// let mut time_message = Message::new();
// read next message size
// read next message type
// if message type == "T"
//     set time_message pos to parser.cursor position
//     

mod message;
mod orderbook;
mod backend;

use message::{
    Message,
    TimeStamp,
    SystemEvent,
    StockDirectory,
    StockTradingAction,
    RegSHORestriction,
    MarketParticipantPosition,
    OrderMessage,
};
use backend::Backend;


struct Parser {
    cursor: Cursor<Vec<u8>>
}

impl Parser {

    fn is_done(&self) -> bool {
        self.cursor.position() as usize == self.cursor.get_ref().len() - 1
    }

    #[inline(always)]
    fn next_message_size(&mut self) -> u16 {
        self.cursor.read_u16::<NetworkEndian>().unwrap()
    }

    #[inline(always)]
    fn next_message_type(&mut self) -> char {
        char::from(self.cursor.read_u8().unwrap())
    }

    #[inline(always)]
    fn skip_message(&mut self, message_size: u16) {
        let mut buf = vec![0; message_size.into()];
        self.cursor.read_exact(&mut buf).unwrap();
    }

    #[inline(always)]
    fn read_string(&mut self, string_length: u16) -> String {
        let mut buf = vec![0; string_length.into()];
        self.cursor.read_exact(&mut buf).unwrap();
        // println!("{:?}", buf);
        String::from_utf8(buf).unwrap()
    }

    fn read_time_stamp(&mut self) -> TimeStamp {
        let seconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        TimeStamp::new( seconds )
    }
    
    fn read_system_event(&mut self, seconds: u32) -> SystemEvent {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let event = char::from(self.cursor.read_u8().unwrap());
        SystemEvent::new( seconds, nanoseconds, event )
    }

    fn read_stock_directory(&mut self, seconds: u32) -> StockDirectory {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let ticker = self.read_string(8);
        let market_category = char::from(self.cursor.read_u8().unwrap());
        let financial_status_indicator = char::from(self.cursor.read_u8().unwrap());
        let round_lot_size = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let round_lots_only = char::from(self.cursor.read_u8().unwrap());
        StockDirectory::new (
            seconds,
            nanoseconds,
            ticker,
            market_category,
            financial_status_indicator,
            round_lot_size,
            round_lots_only,
        )
    }

    fn read_stock_trading_action(&mut self, seconds: u32) -> StockTradingAction {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let ticker = self.read_string(8);
        let trading_state = char::from(self.cursor.read_u8().unwrap());
        let reserved = char::from(self.cursor.read_u8().unwrap());
        let reason = self.read_string(4);

        StockTradingAction::new(
            seconds,
            nanoseconds,
            ticker,
            trading_state,
            reserved,
            reason
        )
    }

    fn read_reg_sho_restriction(&mut self, seconds: u32) -> RegSHORestriction {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let ticker = self.read_string(8);
        let reg_sho_action = char::from(self.cursor.read_u8().unwrap());

        RegSHORestriction::new(seconds, nanoseconds, ticker, reg_sho_action)
    }

    fn read_market_participant_position(&mut self, seconds: u32) -> MarketParticipantPosition {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let mpid = self.read_string(4);
        let ticker = self.read_string(8);
        let primary_market_maker = char::from(self.cursor.read_u8().unwrap());
        let market_maker_mode = char::from(self.cursor.read_u8().unwrap());
        let market_participant_state = char::from(self.cursor.read_u8().unwrap());

        MarketParticipantPosition::new(
            seconds,
            nanoseconds,
            mpid,
            ticker,
            primary_market_maker,
            market_maker_mode,
            market_participant_state
        )
    }

    fn read_add_order(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let side = char::from(self.cursor.read_u8().unwrap());
        let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let ticker = self.read_string(8);
        let price = self.cursor.read_u32::<NetworkEndian>().unwrap();
        
        OrderMessage::new(
            seconds,
            nanoseconds,
            'A',
            refno,
            ticker,
            side,
            shares,
            price,
            None,
            None,
            None,
            None,
        )
    }

    fn read_add_order_with_mpid(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let side = char::from(self.cursor.read_u8().unwrap());
        let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let ticker = self.read_string(8);
        let price = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let mpid = self.read_string(4);
        
        OrderMessage::new(
            seconds,
            nanoseconds,
            'F',
            refno,
            ticker,
            side,
            shares,
            price,
            Some(mpid),
            None,
            None,
            None,
        )
    }

    fn read_execute_order(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let matchno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        
        OrderMessage::new(
            seconds,
            nanoseconds,
            'E',
            refno,
            String::from(""),
            '-',
            shares,
            0,
            None,
            Some(matchno),
            None,
            None,
        )
    }

    fn read_execute_order_with_price(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let matchno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let printable = char::from(self.cursor.read_u8().unwrap());
        let price = self.cursor.read_u32::<NetworkEndian>().unwrap();

        OrderMessage::new(
            seconds,
            nanoseconds,
            'C',
            refno,
            String::from(""),
            '-',
            shares,
            price,
            None,
            Some(matchno),
            Some(printable),
            None,
        )
    }

    fn read_cancel_order(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();

        OrderMessage::new(
            seconds,
            nanoseconds,
            'X',
            refno,
            String::from(""),
            '-',
            shares,
            0,
            None,
            None,
            None,
            None,
        )
    }

    fn read_delete_order(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();

        OrderMessage::new(
            seconds,
            nanoseconds,
            'D',
            refno,
            String::from(""),
            '-',
            0,
            0,
            None,
            None,
            None,
            None,
        )
    }

    fn read_replace_order(&mut self, seconds: u32) -> OrderMessage {
        let nanoseconds = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let new_refno = self.cursor.read_u64::<NetworkEndian>().unwrap();
        let shares = self.cursor.read_u32::<NetworkEndian>().unwrap();
        let price = self.cursor.read_u32::<NetworkEndian>().unwrap();

        OrderMessage::new(
            seconds,
            nanoseconds,
            'U',
            refno,
            String::from(""),
            '_',
            shares,
            price,
            None,
            None,
            None,
            Some(new_refno),
        )        
    }
}

fn main() {
    let file_name = "/Users/colinswaney/GitHub/TotalViewITCH.jl/data/bin/S031413-v41.txt";

    println!("Reading file: {}", file_name);

    let buffer = read(file_name).expect("Unable to read file. Does the file exist?");

    // println!("First 7 bytes: {:?}", &buffer[..7]);
    // println!("Next 8 bytes: {:?}", &buffer[7..15]);
    // println!("Next 7 bytes: {:?}", &buffer[15..22]);

    let cursor = Cursor::new(buffer);
    let mut parser = Parser { cursor };

    let system_messages_writer = csv::Writer::from_path("./data/system_events.csv").unwrap();
    let mut system_messages_bknd = Backend::new( system_messages_writer, 1 );

    let stock_dir_writer = csv::Writer::from_path("./data/stock_directory.csv").unwrap();
    let mut stock_dir_bknd = Backend::new( stock_dir_writer, 10_000);

    let stock_trading_action_writer = csv::Writer::from_path("./data/stock_trading_action.csv").unwrap();
    let mut stock_trading_action_bknd = Backend::new( stock_trading_action_writer, 10_000);

    let reg_sho_writer = csv::Writer::from_path("./data/reg_sho.csv").unwrap();
    let mut reg_sho_bknd = Backend::new( reg_sho_writer, 10_000);

    let market_participant_writer = csv::Writer::from_path("./data/market_participant.csv").unwrap();
    let mut market_participant_bknd = Backend::new( market_participant_writer, 10_000);

    let order_messages_writer = csv::Writer::from_path("./data/order_messages.csv").unwrap();
    let mut order_messages_bknd = Backend::new( order_messages_writer, 10_000);

    let now = Instant::now();
    let mut message_reads = 0;
    let mut seconds: u32 = 0;
    // for _ in 0..100 {
    loop {

        let message_size = parser.next_message_size();
        let message_type = parser.next_message_type();
        match message_type {
            'T' => {
                let message = parser.read_time_stamp();
                seconds = message.get_seconds();
            },
            'S' => {
                let message = parser.read_system_event(seconds);
                message.info();
                // let _ = system_messages_bknd.write(message);
            },
            'R' => {
                let message = parser.read_stock_directory(seconds);
                // let _ = stock_dir_bknd.write(message);
            },
            'H' => {
                let message = parser.read_stock_trading_action(seconds);
                // let _ = stock_trading_action_bknd.write(message);
            },
            'Y' => {
                let message = parser.read_reg_sho_restriction(seconds);
                // let _ = reg_sho_bknd.write(message);
            },
            'L' => {
                let message = parser.read_market_participant_position(seconds);
                // let _ = market_participant_bknd.write(message);
            },
            'A' => {
                let message = parser.read_add_order(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            'F' => {
                let message = parser.read_add_order_with_mpid(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            'E' => {
                let message = parser.read_execute_order(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            'C' => {
                let message = parser.read_execute_order_with_price(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            'X' => {
                let message = parser.read_cancel_order(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            'D' => {
                let message = parser.read_delete_order(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            'U' => {
                let message = parser.read_replace_order(seconds);
                // let _ = order_messages_bknd.write(message);
            },
            _ => {
                // println!("Unrecognized message type {}. Skipping message.", message_type);
                parser.skip_message(message_size - 1);
            }
        }
        message_reads += 1;
        if message_reads % 1000000 == 0 {
            let elapsed = now.elapsed().as_secs_f32();
            println!("elapsed: {}, messages: {}, speed: {}", elapsed, message_reads, message_reads as f32 / elapsed);
        }
        if parser.is_done() {
            break;
        }

        // TODO: implement logic for handling order messages
        
        // if message type in ['A', 'F']
            // if message.ticker in tickers
                // add order to order list
                // update order book
        
        // if message type in ['E', 'C', 'X', 'D']
            // if message.ticker in tickers
                // complete the message
                // update the order list
                // updaet the order book

        // if message.message_type == ['U']
            // message.complete(&orders);
            // if message.ticker in tickers
                // delete_msg, add_msg = message.split();
                // add_msg.complete(&orders);
                // delete_msg.complete(&orders);
                // order_messages_bknd.write(&message);
                // message_writes += 1;
                // orders.update(&delete_msg);
                // books.update(&delete_msg);
                // orders.add(&add_msg);
                // books.update(&add_msg);
                // order_books_bknd.write(books)



        
        // TODO: implement logic for writing to disk
        // 

    }
}
