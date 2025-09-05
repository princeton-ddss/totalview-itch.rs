# Lobste.rs
A command-line tool to process NASDAQ TotalView-ITCH data for academic research.

## Description
Nasdaq TotalView-ITCH (“TotalView”) is a data feed used by professional traders to maintain a real-time view of market conditions. TotalView disseminates all quote and order activity for securities traded on the Nasdaq exchange—several billion messages per day—allowing users to reconstruct the limit order book for any security up to arbitrary depth with nanosecond precision. It is a unique data source for financial economists and engineers examining topics such as information flows through lit exchanges, optimal trading strategies, and the development of macro-level indicators from micro-level signals (e.g., a market turbulence warning).

While TotalView data is provided at no charge to academic researchers via the Historical TotalView-ITCH offering, the historical data offering uses a binary file specification that poses challenges for researchers. Lobste.rs is a pure Rust package developed to efficiently process and store historical data files for academic research purposes.

## Getting Started

### Installation
Coming soon!

### Basic Usage
Usage is straightforward:
```shell
lobsters data/S031413-v41.txt --tickers AAPL
```
This example parses a raw ITCH file, `S031413-v41.txt`, which happens to have
`v4.1` formatting, and stores the extracted data for the specified ticker (message, orderbooks,
etc.) to CSV files in `./data`. To process multiple tickers, simply add the additional tickers to the
list:
```shell
lobsters data/S031413-v41.txt --tickers AAPL,MSFT
```
Processing of multiple files (i.e., dates) can be performed using multiple processes or multiple
jobs on a high-performance computing cluster.

The processed data can be loaded using your favorite data processing tools, e.g., Polars.

<!-- > [!TIP]
> For large-scale analyses, its recommended to convert the processed data to
> the Apache Parquet format and use tools such as Apache Spark. -->

## Storage
Lobste.rs aims to support a variety data storage options. We currently support writing to CSV and 
aim to support Parquet and Postgres in the need future.

### CSV
The default writer stores data in CSV format. Output has the following directory structure:
```shell
test
|- messages
   |- ticker=A
      |- date=2013-03-14
         |- partition.csv
|- orderbooks
|- noii
|- trades
```
This structure is convenient for parallelizing analyses performed at the
ticker-date level. 

### Postgres
Coming soon 🦺 🚧 🔨

### Parquet
Coming soon 🦺 🚧 🔨

## Data
The default parsing method creates four tables/collections:

- `messages`: messages that reflect order book updates,
- `orderbooks`: order book snapshots following each message, 
- `noii`: net order imbalance indicator messages, 
- `trades`: messages that indicate trades involving non-displayed orders, 

All records are stored in ascending temporal order, and all data is stored without
modification, i.e., all fields adhere to the format described in the
relevant TotalView specification.

### `messages`
Each row of the `messages` table indicates an update to the order book. The types of updates are:

- Add (`A` or `F`)
- Cancel (`X`)
- Delete (`D`)
- Replace (`U`)
- Execute (`E` or `C`)

Note that replace orders are **not** split into their constituent add and delete orders in the database.

| Field    | Type     | Description                                                             | Required? | Default   |
| -------- | -------- | ----------------------------------------------------------------------- | :-------: | :-------: |
| date     | `Date`   | The file date (`YYYY-MM-DD`).                                           | ✓         |           |
| sec      | `Int`    | The number of seconds since midnight.                                   | ✓         |           |
| nano     | `Int`    | The number of nanoseconds since the most recent second.                 | ✓         |           |
| type     | `Char`   | The message type symbol as defined in TotalView specification.          | ✓         |           |
| ticker   | `String` | The stock ticker associated with the message.                           | ✓         |           |
| side     | `Char`   | The side of the order book affected by the message (`B` or `S`).        | ✓         |           |
| price    | `Int`    | The price associated with an order update.                              | ✓         |           |
| refno    | `Int`    | A day-unique reference number associated with an original limit order.  | ✓         |           |
| newrefno | `Int`    | A day-unique reference number associated with a new limit order.        |           | `Missing` |
| mpid     | `String` | An optional market participant identifier.                              |           | `Missing` |

### `orderbooks`
Each row the `orderbooks` table represents a snapshot of the order book associated with an order book update. That is, the `n`-th row of the `orderbooks` table represents the state of the order book immediately following the update indicated by the `n`-th row of the `messages` table. The exact fields available depend on the number of levels of levels tracked during parsing, `N`. For a given `N`, prices and shares are recorded in order from best to worst offer for bids and asks, respectively.

| Field          | Type   | Description                                                     | Required?   | Default   |
| -------------- | ------ | --------------------------------------------------------------- | :---------: | :-------: |
| date           | `Date` | The file date (`YYYY-MM-DD`).                                   | ✓           |           |
| sec            | `Int`  | The number of seconds since midnight.                           | ✓           |           |
| nano           | `Int`  | The number of nanoseconds since the most recent second.         | ✓           | `Missing` |
| bid_price_`n`  | `Int`  | The offer price of the `n`-th best bid (`N=1,..., N`).          | ✓           | `Missing` |
| ask_price_`n`  | `Int`  | The offer price of the `n`-th best ask (`N=1,..., N`).          | ✓           | `Missing` |
| bid_shares_`n` | `Int`  | The offer volume at the `n`-th best bid (`N=1,..., N`).         | ✓           | `Missing` |
| ask_shares_`n` | `Int`  | The offer volume at the `n`-th best ask (`N=1,..., N`).         | ✓           | `Missing` |

### `noii`
Net Order Imbalance Indicator (NOII) messages are disseminated prior to market open and close as well as during quote only periods. The `noii` collection stores these messages for all tickers in a single file for each date.

| Field      | Type     | Description                                                     | Required? | Default   |
| ---------- | -------- | --------------------------------------------------------------- | :-------: | :-------: |
| date       | `Date`   | The file date (`YYYY-MM-DD`).                                   | ✓         |           |
| sec        | `Int`    | The number of seconds since midnight.                           | ✓         |           |
| nano       | `Int`    | The number of nanoseconds since the most recent second.         | ✓         |           |
| type       | `Char`   | The cross type: opening (`O`), close (`C`) or halted (`H`).     | ✓         |           |
| ticker     | `String` | The stock ticker associated with the message.                   | ✓         |           |
| paired     | `Int`    | The number of shares matched at the current reference price.    | ✓         |           |
| imbalance  | `Int`    | The number of shares not paired at the current reference price. | ✓         |           |
| direction  | `Char`   | The side of the imbalance (`B`, `S`, `N` or `O`).               | ✓         |           |
| far        | `Int`    | A hypothetical clearing price for cross orders only.            | ✓         |           |
| near       | `Int`    | A hypothetical clearing price for cross and continuous orders.  | ✓         |           |
| current    | `Int`    | The price at which the imbalance is calculated.                 | ✓         |           |

### `trades`
Rows of the `trades` collection reflect two types of trades that are not captured in the order book update: cross and non-cross trades. Non-cross trade messages "provide details for normal match events involving non-displayable order type"—i.e., hidden orders. Cross trade message (`type=='Q'`) "indicate that Nasdaq has completed its cross process for a specific security". Neither trade type affects the state of the (visible) order book, but both should be included in volume calculations.

| Field   | Type     | Description                                                                | Required?           | Default   |
| ------- | -------- | -------------------------------------------------------------------------- | :-----------------: | --------- |
| date    | `Date`   | The file date (`YYYY-MM-DD`).                                              | ✓                   |           |
| sec     | `Int`    | The number of seconds since midnight.                                      | ✓                   |           |
| nano    | `Int`    | The number of nanoseconds since the most recent second.                    | ✓                   |           |
| type    | `Char`   | The type of trade: hidden (`P`) or cross (`Q`).                            | ✓                   |           |
| ticker  | `String` | The stock ticker associated with the trade.                                | ✓                   |           |
| refno   | `Int`    | A day-unique reference number associated with an original limit order.     | Hidden trades only. | `Missing` |
| matchno | `Int`    | A day-unique reference number associated with the trade or cross.          | ✓                   |           |
| side    | `Char`   | The type of non-display order matched (`B` of `S`).                        | Hidden trades only. | `Missing` |
| price   | `Int`    | The price of the cross.                                                    | Cross trades only.  | `Missing` |
| shares  | `Int`    | The number of shares traded.                                               | ✓                   |           |
| cross   | `Int`    | The cross type: opening (`O`), close (`C`), halted (`H`) or intrday (`I`). | ✓                   |           |


## Data Version Support
`Lobste.rs` supports versions `4.1` and `5.0` of the TotalView-ITCH file
specificiation. The parser processes all message types required to reconstruct
limit order books as well as several types that do not impact the order book.

| Message Type       | Symbol | Supported? | Notes                                 |
| ------------------ | :----: | :--------: | ------------------------------------- |
| Timestamp          | T      | 4.1        | Message type only exists for `v4.1`.  |
| System             | S      | ✓          |                                       |
| Market Participant | L      |            |                                       |
| Trade Action       | H      | ✓          |                                       |
| Reg SHO            | Y      |            |                                       |
| Stock Directory    | R      |            |                                       |
| Add                | A      | ✓          |                                       |
| Add w/ MPID        | F      | ✓          |                                       |
| Execute            | E      | ✓          |                                       |
| Execute w/ Price   | C      | ✓          |                                       |
| Cancel             | X      | ✓          |                                       |
| Delete             | D      | ✓          |                                       |
| Replace            | U      | ✓          |                                       |
| Cross Trade        | Q      | ✓          | Ignored by order book updates.        |
| Trade              | P      | ✓          | Ignored by order book updates.        |
| Broken Trade       | B      |            | Ignored by order book updates.        |
| NOII               | I      | ✓          |                                       |
| RPII               | N      |            |                                       |


#### Planned Work
We plan to process and record the following additional message types:
- stock related messages (e.g., financial status and market category),
- stock trading action codes (e.g., trading halts for individual stocks),
- Reg SHO codes,
- market participant position codes,
- execution codes

> [!WARNING]
> Note that the format of the database is not stable and will likely change in the near future.

#### Not Planned
There are no plans to support the following message categories:
- broken trade messages (4.6.3)
- retail price improvement indicator (RPII) messages (4.8),
- market-wide circuit breaker messages (4.2.5)
- IPO quoting period updates (4.2.6),
- Limit up/down (LULD) aution collar messages (4.2.7),
- Operational halt messages (4.2.8),


<!-- #### System Event Codes
There is no additional processing required for daily system event codes except for variant "C", which indicates the end of messages and therefore signals the program to stop reading messages. Likewise, there is no special processing required for system event codes that indicate emergency market conditions. We simply record these messages in the messages database. -->

<!-- #### Stock Trading Action Codes
- I don't know what should be done about these codes. E.g. if a stock is halted or paused, then does Nasdaq disseminate messages for that stock (that need to be ignored until resumption)? In that case, I can simply record the message. Otherwise, I have to hold onto incoming messages for that stock until resumption and then run order book updates on the backlog before processing new messages.

#### Reg SHO Codes
- No idea...

#### Market Participant Position Codes
- These can simply be recorded. They have no impact on the order book.

#### Execution Codes
- The printable code ('Y' or 'N') has no effect on order book updates, but should be recorded in the database for volume calculations. -->

## Contributing
This package is intended to be a community resource for researchers working with
TotalViewITCH. If you find a bug, have a suggestion or otherwise wish to
contribute to the package, please feel free to create an issue.
