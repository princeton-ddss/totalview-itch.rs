use std::io::Error;

use crate::Parser;
use byteorder::{NetworkEndian, ReadBytesExt};

pub trait Message {

    fn len(&self) -> u64;

    fn pos(&self) -> u64;

    fn set_position(&mut self, pos: u64) ;

    fn skip(&self, parser: &mut Parser) {
        parser.cursor.set_position(self.pos() + self.len());
    }
}

pub struct TimeMessage {
    pos: u64, // position 0 => cursor pointing to the first byte of the message, excluding size and type bytes
}

impl TimeMessage {

    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn seconds(&self, parser: &mut Parser) -> Result<u32, Error> {
        parser.cursor.read_u32::<NetworkEndian>()
    }
}

impl Message for TimeMessage {

    fn len(&self) -> u64 {
        4
    }

    fn pos(&self) -> u64 {
        return self.pos
    }

    fn skip(&self, parser: &mut Parser) {
        parser.cursor.set_position(self.pos + self.len());
    }

    fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }


}

pub struct SystemMessage {
    pos: u64,
}

impl SystemMessage {

    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> Result<u32, Error> {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>()
    }

    pub fn event_code(&self, parser: &mut Parser) -> Result<u8, Error> {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u8()
    }
}

impl Message for SystemMessage {
    fn len(&self) -> u64 {
        5
    }

    fn pos(&self) -> u64 {
        return self.pos
    }

    fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

pub struct StockDirectoryMessage {
    pos: u64,
}

impl StockDirectoryMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }

    pub fn ticker(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4);
        parser.read_string(8)
    }

    pub fn market_category(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }

    pub fn financial_status_indicator(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8 + 1);
        char::from(parser.cursor.read_u8().unwrap())
    }

    pub fn round_lot_size(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 1);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }

    pub fn round_lots_only(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 1 + 4);
        char::from(parser.cursor.read_u8().unwrap())
    }

}

impl Message for StockDirectoryMessage {
    fn len(&self) -> u64 {
        19
    }

    fn pos(&self) -> u64 {
        return self.pos
    }

    fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

pub struct StockTradingActionMessage {
    pos: u64,
}

impl StockTradingActionMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    pub fn ticker(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4);
        parser.read_string(8)
    }
    pub fn trading_state(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }
    pub fn reserved(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8 + 1);
        char::from(parser.cursor.read_u8().unwrap())
    }
    pub fn reason(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 1);
        parser.read_string(4)
    }
}

impl Message for StockTradingActionMessage {
    fn len(&self) -> u64 {
        18
    }

    fn pos(&self) -> u64 {
        self.pos
    }
    
    fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

pub struct RegSHOMessage {
    pos: u64,
}

impl RegSHOMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }

    pub fn ticker(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4);
        parser.read_string(8)
    }

    pub fn reg_sho_action(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }
}

impl Message for RegSHOMessage {
    fn len(&self) -> u64 {
        13
    }

    fn pos(&self) -> u64 {
        self.pos
    }
    
    fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

pub struct MarketParticipantMessage {
    pos: u64,
}

impl MarketParticipantMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }

    pub fn mpid(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4);
        parser.read_string(4)
    }

    pub fn ticker(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4 + 4);
        parser.read_string(8)
    }

    pub fn primary_market_maker(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }

    pub fn market_maker_mode(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 4 + 8 + 1);
        char::from(parser.cursor.read_u8().unwrap())
    }

    pub fn market_participant_state(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 4 + 8 + 1 + 1);
        char::from(parser.cursor.read_u8().unwrap())
    }
}

impl Message for MarketParticipantMessage {
    fn len(&self) -> u64 {
        19
    }
    fn pos(&self) -> u64 {
        self.pos
    }
    
    fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

pub struct AddOrderMessage {
    pos: u64,
}

impl AddOrderMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }
    
    pub fn side(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }
    
    pub fn shares(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 1);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
        
    }
    
    pub fn ticker(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 4);
        parser.read_string(8)
    }
    
    pub fn price(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 4 + 8);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }

}

pub struct AddOrderWithMPIDMessage {
    pos: u64,
}

impl AddOrderWithMPIDMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }
    
    pub fn side(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }
    
    pub fn shares(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 1);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
        
    }
    
    pub fn ticker(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 4);
        parser.read_string(8)
    }
    
    pub fn price(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 4 + 8);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn mpid(&self, parser: &mut Parser) -> String {
        parser.cursor.set_position(self.pos + 4 + 8 + 1 + 4 + 8 + 4);
        parser.read_string(4)
    }
}

pub struct ExecuteOrderMessage {
    pos: u64,
}

impl ExecuteOrderMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }
    
    pub fn shares(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
        
    }

    pub fn matchno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4 + 8 + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
    }

}

pub struct ExecuteOrderWithPriceMessage {
    pos: u64,
}

impl ExecuteOrderWithPriceMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }
    
    pub fn shares(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
        
    }

    pub fn matchno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4 + 8 + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
    }

    pub fn printable(&self, parser: &mut Parser) -> char {
        parser.cursor.set_position(self.pos + 4 + 8 + 4 + 8);
        char::from(parser.cursor.read_u8().unwrap())
    }
    
    pub fn price(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 4 + 8 + 1);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
}

pub struct CancelOrderMessage {
    pos: u64,
}

impl CancelOrderMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }
    
    pub fn shares(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
        
    }

}

pub struct DeleteOrderMessage {
    pos: u64,
}

impl DeleteOrderMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }

}

pub struct ReplaceOrderMessage {
    pos: u64,
}

impl ReplaceOrderMessage {

    pub fn new() -> Self {
        Self { pos: 0}
    }

    pub fn nanoseconds(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }
    
    pub fn refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }
    
    pub fn new_refno(&self, parser: &mut Parser) -> u64 {
        parser.cursor.set_position(self.pos + 4 + 8);
        parser.cursor.read_u64::<NetworkEndian>().unwrap()
        
    }

    pub fn shares(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 8);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
        
    }

    pub fn price(&self, parser: &mut Parser) -> u32 {
        parser.cursor.set_position(self.pos + 4 + 8 + 8 + 4);
        parser.cursor.read_u32::<NetworkEndian>().unwrap()
    }

}

pub struct TradeMessage {
    pos: u64,
}

impl TradeMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }
}

pub struct CrossTradeMessage {
    pos: u64,
}

impl CrossTradeMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }
}

pub struct NOIIMessage {
    pos: u64,
}

impl NOIIMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }
}

pub struct  RPIIMessage {
    pos: u64,
}

impl RPIIMessage {
    pub fn new() -> Self {
        Self { pos: 0 }
    }
}

