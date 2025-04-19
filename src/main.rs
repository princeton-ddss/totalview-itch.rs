use lobsters::{Buffer, Parser, Version};

fn main() {
    let tickers = ["SHV", "TEF", "STM"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // let mut buffer = Buffer::new("./data/bin/S031413-v41.txt");
    // let parser = Parser::new(Version::V41, tickers);
    let mut buffer = Buffer::new("./data/bin/S022717-v50.txt");
    let parser = Parser::new(Version::V50, tickers);

    for _ in 0..100 {
        let msg = parser.extract_message(&mut buffer).unwrap();
        println!("\n{:?}", msg);
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
