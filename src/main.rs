use std::fs::read;
use std::io::{Cursor, Read};
use byteorder::{NetworkEndian, ReadBytesExt};
use std::time::Instant;
use std::io::{Seek, SeekFrom};

// mod message;
// mod orderbook;
mod messages;

use messages::{
    Message,
    TimeMessage,
    SystemMessage,
    StockDirectoryMessage,
    StockTradingActionMessage,
    RegSHOMessage,
    MarketParticipantMessage,
    AddOrderMessage,
    AddOrderWithMPIDMessage,
    ExecuteOrderMessage,
    ExecuteOrderWithPriceMessage,
    CancelOrderMessage,
    DeleteOrderMessage,
    ReplaceOrderMessage,
    TradeMessage,
    CrossTradeMessage,
    NOIIMessage,
    RPIIMessage
};

struct Parser {
    cursor: Cursor<Vec<u8>>
}

impl Parser {

    fn is_done(&self) -> bool {
        self.cursor.position() as usize == self.cursor.get_ref().len() - 1
    }

    #[inline(always)]
    fn get_next_message_size(&mut self) -> u16 {
        self.cursor.read_u16::<NetworkEndian>().unwrap()
    }

    #[inline(always)]
    fn get_next_message_type(&mut self) -> char {
        char::from(self.cursor.read_u8().unwrap())
    }

    #[inline(always)]
    fn skip_message(&mut self, message_size: u16) {
        self.cursor.seek(SeekFrom::Current(message_size.into())).unwrap();
    }

    #[inline(always)]
    fn read_string(&mut self, string_length: u16) -> String {
        // TODO: can you re-use a String here?
        let mut buf = vec![0; string_length.into()];
        self.cursor.read_exact(&mut buf).unwrap();
        // println!("{:?}", buf);
        String::from_utf8(buf).unwrap()
    }
}

fn main() {
    let file_name = "/Users/colinswaney/GitHub/TotalViewITCH.jl/data/bin/S031413-v41.txt";

    println!("Reading file: {}", file_name);

    let buffer = read(file_name).expect("Unable to read file. Does the file exist?");

    let cursor = Cursor::new(buffer);
    let mut parser = Parser { cursor };

    let mut time_msg = TimeMessage::new();
    let mut system_msg = SystemMessage::new();
    let mut stock_dir_msg = StockDirectoryMessage::new();
    let mut stock_action_msg = StockTradingActionMessage::new();
    let mut reg_sho_msg = RegSHOMessage::new();
    let mut market_part_msg = MarketParticipantMessage::new();
    // Have to decide how to handle order messages. It seems to make sense to
    // keep them separate in this framework. Try to capture commonality through
    // a trait (e.g., trait Order).
    let mut add_order_msg = AddOrderMessage::new();
    let mut add_order_mpid_msg = AddOrderWithMPIDMessage::new();
    let mut execute_order_msg = ExecuteOrderMessage::new();
    let mut execute_order_with_price_msg = ExecuteOrderWithPriceMessage::new();
    let mut cancel_order_msg = CancelOrderMessage::new();
    let mut delete_order_msg = DeleteOrderMessage::new();
    let mut replace_order_msg = ReplaceOrderMessage::new();
    let mut trade_message = TradeMessage::new();
    let mut cross_trade_message = CrossTradeMessage::new();
    let mut noii_message = NOIIMessage::new();
    let mut rpii_message = RPIIMessage::new();

    let now = Instant::now();
    let mut message_reads = 0;
    let mut seconds: u32;
    for _ in 0..10_000_000 {
    // loop {
        let message_size = parser.get_next_message_size();
        let message_type = parser.get_next_message_type();
        // TODO: use enum instead of char for message type matching
        match message_type {
            'T' => {
                time_msg.set_position(parser.cursor.position());
                seconds = time_msg.seconds(&mut parser).unwrap();
                if seconds % 1800 == 0 {
                    println!("The time is: {}", seconds);
                }
                time_msg.skip(&mut parser);
            },
            'S' => {
                system_msg.set_position(parser.cursor.position());
                let event_code = char::from(system_msg.event_code(&mut parser).unwrap());
                println!("System event: {}", event_code);
                system_msg.skip(&mut parser);
            },
            'R' => {
                stock_dir_msg.set_position(parser.cursor.position());
                // TODO: do something with the message...
                stock_dir_msg.skip(&mut parser);
            },
            'H' => {
                stock_action_msg.set_position(parser.cursor.position());
                stock_action_msg.skip(&mut parser);
            },
            'Y' => {
                reg_sho_msg.set_position(parser.cursor.position());
                reg_sho_msg.skip(&mut parser);
            },
            'L' => {
                market_part_msg.set_position(parser.cursor.position());
                market_part_msg.skip(&mut parser);
            },
            _ => {
                parser.skip_message(message_size - 1);
            },
        }
        message_reads += 1;
        if message_reads % 1_000_000 == 0 {
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
