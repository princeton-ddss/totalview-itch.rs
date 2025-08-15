use lobsters::{message::IntoOrderMessage, BufFile, Message, Parser, Version, Writer, CSV};

fn main() {
    let mut buffer = BufFile::with_capacity(1024, "data/S022717-v50.txt").unwrap();

    let tickers = ["SHV", "TEF", "STM"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut parser = Parser::new(Version::V50, tickers);

    let backend = CSV::new("../collections").unwrap();
    let mut writer = Writer::<10, CSV>::new(backend);

    let date = String::from("2017-02-27"); // To be derived from the file name

    for _ in 0..100 {
        let msg = parser.extract_message(&mut buffer).unwrap();

        println!("\n{:?}", msg);

        match msg {
            Message::AddOrder(data) => {
                let order_message = data.into_order_message(date.clone());
                writer.write_order_message(order_message).unwrap();
            }
            Message::CancelOrder(data) => {
                let order_message = data.into_order_message(date.clone());
                writer.write_order_message(order_message).unwrap();
            }
            Message::DeleteOrder(data) => {
                let order_message = data.into_order_message(date.clone());
                writer.write_order_message(order_message).unwrap();
            }
            Message::ExecuteOrder(data) => {
                let order_message = data.into_order_message(date.clone());
                writer.write_order_message(order_message).unwrap();
            }
            _ => {}
        }
    }
}
