//use std::time::Duration;
//use std::sync::{Arc, Mutex};
//use std::thread;
use tungstenite::{connect, Message};
//use url::Url;
//use serde_json::{Value};

fn main() {
    let req = r#"
        {
            "method": "public/subscribe",
            "params": {
                "channels": [
                    "book.BTC-PERPETUAL.100ms"
                ]
            },
            "jsonrpc": "2.0",
            "id": 0
        }"#.to_string();

    let (mut socket, _response) = connect("wss://test.deribit.com/ws/api/v2/").expect("Connection error");
    socket.write_message(Message::Text(req)).unwrap();

    loop {
        println!(" ");
        let msg = socket.read_message().expect("Reading error");
        dbg!(msg);
    }

}
