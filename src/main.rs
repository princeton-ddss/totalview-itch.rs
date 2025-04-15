mod parser;

fn main() {
    let mut parser = parser::Parser::new("./data/bin/S031413-v41.txt");
    for _ in 0..100 {
        parser.next().unwrap();
        println!("\n{:?}", parser.get_current_message());
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
