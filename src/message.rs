use crate::orderbook::Order;
use csv::Error;
use strum_macros::EnumString;
use serde::{Serialize, ser::SerializeStruct};

// What if you created a Message type that you could use to construct specific
// messages types from by providing a list of field names and types.
// Then you could write a single read_message method that would just iteratre
// through the fields. The "problem" with this is that you would need to
// check every messages type from its field instead of using a Rust type.

#[derive(Debug, PartialEq, EnumString)]
pub enum SystemEventCode {
    #[strum(serialize = "O")]
    StartMessages,
    #[strum(serialize = "S")]
    StartSystem,
    #[strum(serialize = "Q")]
    StartMarketHours,
    #[strum(serialize = "M")]
    EndMarketHours,
    #[strum(serialize = "E")]
    EndSystem,
    #[strum(serialize = "C")]
    EndMessages,
    #[strum(serialize = "A")]
    EmergencyMarketHalt,
    #[strum(serialize = "R")]
    EmergencyMarketQuoteOnly,
    #[strum(serialize = "B")]
    EmergencyMarketResumption,
}

enum FinancialStatusIndicatorCode {
    // TODO
}

enum TradingStateCode {
    // TODO
}

enum RegSHOActionCode {
    // TODO
}

#[derive(EnumString)]
pub enum Side {
    #[strum(serialize = "B")]
    Buy, // "B"
    #[strum(serialize = "S")]
    Sell, // "S"
}

pub trait Message {
    fn info(&self) {
        let symbol = self.get_symbol();
        let seconds = self.get_seconds();
        let nanoseconds = self.get_nanoseconds();
        println!("Message type {} received at time {}.{}", symbol, seconds, nanoseconds);
    }

    fn get_symbol(&self) -> char;
    
    fn get_seconds(&self) -> u32;

    fn get_nanoseconds(&self) -> u32;
}

pub trait AddOrderMessage {
    fn to_order(&self) -> Order;
    fn get_refno(&self) -> u64;
    fn get_shares(&self) -> u32;
    fn get_price(&self) -> u32;
    fn get_side(&self) -> char;
}

pub trait ModifyOrderMessage {
    fn get_refno(&self) -> u64;
    fn get_side(&self) -> char;
    fn get_shares(&self) -> u32;
    fn get_price(&self) -> u32;
}

pub struct TimeStamp {
    seconds: u32
}

impl TimeStamp {
    pub fn new(seconds: u32) -> Self {
        Self { seconds }
    }
}

impl Message for TimeStamp {
    fn get_symbol(&self) -> char { 'T' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { 0 }
}

#[derive(Serialize)]
pub struct SystemEvent {
    seconds: u32,
    nanoseconds: u32,
    // event: SystemEventCode,
    event: char,
}

impl SystemEvent {
    pub fn new(seconds: u32, nanoseconds: u32, event: char) -> Self {
        Self { seconds, nanoseconds, event }
    }
}

impl Message for SystemEvent {
    fn get_symbol(&self) -> char { 'S' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct StockDirectory {
    seconds: u32,
    nanoseconds: u32,
    ticker: String,
    market_category: char,
    // financial_status_indicator: FinancialStatusIndicatorCode,
    financial_status_indicator: char,
    round_lot_size: u32,
    // round_lots_only: bool,
    round_lots_only: char,
}

impl StockDirectory {
    pub fn new(seconds: u32, nanoseconds: u32, ticker: String, market_category: char, financial_status_indicator: char, round_lot_size: u32, round_lots_only: char) -> Self {
        Self { seconds, nanoseconds, ticker, market_category, financial_status_indicator, round_lot_size, round_lots_only }
    }
}

impl Message for StockDirectory {
    fn get_symbol(&self) -> char { 'R' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct StockTradingAction {
    seconds: u32,
    nanoseconds: u32,
    ticker: String,
    // trading_state: TradingStateCode,
    trading_state: char,
    reserved: char,
    reason: String,
}

impl StockTradingAction {
    pub fn new(seconds: u32, nanoseconds: u32, ticker: String, trading_state: char, reserved: char, reason: String) -> Self {
        Self { seconds, nanoseconds, ticker, trading_state, reserved, reason }
    }
}

impl Message for StockTradingAction {
    fn get_symbol(&self) -> char { 'H' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct RegSHORestriction {
    seconds: u32,
    nanoseconds: u32,
    ticker: String,
    // reg_sho_action: RegSHOActionCode,
    reg_sho_action: char,
}

impl RegSHORestriction {
    pub fn new(seconds: u32, nanoseconds: u32, ticker: String, reg_sho_action:char) -> Self {
        Self { seconds, nanoseconds, ticker, reg_sho_action }
    }
}

impl Message for RegSHORestriction {
    fn get_symbol(&self) -> char { 'Y' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct MarketParticipantPosition {
    seconds: u32,
    nanoseconds: u32,
    mpid: String, // 4
    ticker: String, // 8
    primary_market_maker: char,
    market_maker_mode: char,
    market_participant_state: char,
}

impl MarketParticipantPosition {
    pub fn new(
        seconds: u32,
        nanoseconds: u32,
        mpid: String,
        ticker: String,
        primary_market_maker: char,
        market_maker_mode: char,
        market_participant_state: char,
    ) -> Self {
        Self {
            seconds,
            nanoseconds,
            mpid,
            ticker,
            primary_market_maker,
            market_maker_mode,
            market_participant_state
        }
    }
}

impl Message for MarketParticipantPosition {
    fn get_symbol(&self) -> char { 'L' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

enum OrderMessageType {
    AddOrder,
    AddOrderWithMPID,
    ExecuteOrder,
    ExecuteOrderWithPrice,
    CancelOrder,
    DeleteOrder,
    ReplaceOrder,
}

#[derive(Serialize)]
pub struct OrderMessage {
    seconds: u32,
    nanoseconds: u32,
    message_type: char,
    refno: u64,
    ticker: String,
    side: char,
    shares: u32,
    price: u32,
    mpid: Option<String>,
    matchno: Option<u64>,
    printable: Option<char>,
    new_refno: Option<u64>,
}

impl OrderMessage {

    pub fn new(seconds: u32, nanoseconds: u32, message_type: char, refno: u64, ticker: String, side: char, shares: u32, price: u32, mpid: Option<String>, matchno: Option<u64>, printable: Option<char>, new_refno: Option<u64>) -> Self {
        Self {
            seconds,
            nanoseconds,
            message_type,
            refno,
            ticker,
            side,
            shares,
            price,
            mpid,
            matchno,
            printable,
            new_refno,
        }
    }

    // pub fn to_order(&self) -> Result<Order> {
    //     if self.message_type == 'A' || self.message_type == 'F' {
    //         Ok(Order::new(self.refno, self.ticker.clone(), self.side, self.shares, self.price))
    //     } else {
    //         // Error
    //     }
    // }
}

impl Message for OrderMessage {
    fn get_symbol(&self) -> char { self.message_type }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}


// #[derive(Serialize)]
pub struct AddOrder {
    seconds: u32,
    nanoseconds: u32,
    refno: u64,
    // side: Side,
    side: char,
    shares: u32,
    ticker: String, // 8 bytes
    price: u32,
}

impl AddOrder {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64, side: char, shares: u32, ticker: String, price: u32) -> Self {
        Self { seconds, nanoseconds, refno, side, shares, ticker, price }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }

    pub fn get_price(&self) -> u32 {
        self.price
    }

    pub fn get_shares(&self) -> u32 {
        self.shares
    }

    pub fn to_order(&self) -> Order {
        Order::new(self.refno, self.ticker.clone(), self.side, self.shares, self.price)
    }
}

impl Serialize for AddOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("AddOrder", 12)?;
        state.serialize_field("seconds", &self.seconds)?;
        state.serialize_field("nanoseconds", &self.nanoseconds)?;
        state.serialize_field("message_type", &'A')?;
        state.serialize_field("refno", &self.refno)?;
        state.serialize_field("new_refno", "")?;
        state.serialize_field("ticker", &self.ticker)?;
        state.serialize_field("side", &self.side)?;
        state.serialize_field("shares", &self.shares)?;
        state.serialize_field("price", &self.price)?;
        state.serialize_field("mpid", "")?;
        state.serialize_field("matchno", "")?;
        state.serialize_field("printable", "")?;
        state.end()
    }
}

impl Message for AddOrder {
    fn get_symbol(&self) -> char { 'A' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

pub struct AddOrderWithMPID {
    seconds: u32,
    nanoseconds: u32,
    refno: u64,
    // side: Side,
    side: char,
    shares: u32,
    ticker: String,
    price: u32,
    mpid: String, // 4 bytes        
}

impl AddOrderWithMPID {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64, side: char, shares: u32, ticker: String, price: u32, mpid: String) -> Self {
        Self { seconds, nanoseconds, refno, side, shares, ticker, price, mpid }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }

    pub fn get_price(&self) -> u32 {
        self.price
    }

    pub fn get_shares(&self) -> u32 {
        self.shares
    }    
}

impl Message for AddOrderWithMPID {
    fn get_symbol(&self) -> char { 'F' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

impl Serialize for AddOrderWithMPID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("AddOrderWithMPID", 12)?;
        state.serialize_field("seconds", &self.seconds)?;
        state.serialize_field("nanoseconds", &self.nanoseconds)?;
        state.serialize_field("message_type", &'F')?;
        state.serialize_field("refno", &self.refno)?;
        state.serialize_field("new_refno", "")?;
        state.serialize_field("ticker", &self.ticker)?;
        state.serialize_field("side", &self.side)?;
        state.serialize_field("shares", &self.shares)?;
        state.serialize_field("price", &self.price)?;
        state.serialize_field("mpid", &self.mpid)?;
        state.serialize_field("matchno", "")?;
        state.serialize_field("printable", "")?;
        state.end()
    }
}


pub struct ExecuteOrder {
    seconds: u32,
    nanoseconds: u32,
    refno: u64,
    shares: u32,
    matchno: u64,
}

impl ExecuteOrder {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64, shares: u32, matchno: u64) -> Self {
        Self { seconds, nanoseconds, refno, shares, matchno }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }
}

impl Message for ExecuteOrder {
    fn get_symbol(&self) -> char { 'E' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}


#[derive(Serialize)]
pub struct ExecuteOrderWithPrice {
    seconds:u32,
    nanoseconds: u32,
    refno: u64,
    shares: u32,
    matchno: u64,
    printable: char, // "Y" or "N"
    price: u32,
}

impl ExecuteOrderWithPrice {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64, shares: u32, matchno: u64, printable: char, price: u32) -> Self {
        Self { seconds, nanoseconds, refno, shares, matchno, printable, price }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }
}

impl Message for ExecuteOrderWithPrice {
    fn get_symbol(&self) -> char { 'C' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct CancelOrder {
    seconds: u32,
    nanoseconds: u32,
    refno: u64,
    shares: u32,
}

impl CancelOrder {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64, shares: u32) -> Self {
        Self { seconds, nanoseconds, refno, shares }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }
}

impl Message for CancelOrder {
    fn get_symbol(&self) -> char { 'X' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct DeleteOrder {
    seconds: u32,
    nanoseconds: u32,
    refno: u64,
}

impl DeleteOrder {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64) -> Self {
        Self { seconds, nanoseconds, refno }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }
}

impl Message for DeleteOrder {
    fn get_symbol(&self) -> char { 'D' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}

#[derive(Serialize)]
pub struct ReplaceOrder {
    seconds: u32,
    nanoseconds: u32,
    refno: u64,
    new_refno: u64,
    shares: u32,
    price: u32,
}

impl ReplaceOrder {
    pub fn new(seconds: u32, nanoseconds: u32, refno: u64, new_refno: u64, shares: u32, price: u32) -> Self {
        Self { seconds, nanoseconds, refno, new_refno, shares, price }
    }

    pub fn get_refno(&self) -> u64 {
        self.refno
    }
}

impl Message for ReplaceOrder {
    fn get_symbol(&self) -> char { 'U' }
    fn get_seconds(&self) -> u32 { self.seconds }
    fn get_nanoseconds(&self) -> u32 { self.nanoseconds }
}