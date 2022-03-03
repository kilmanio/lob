//use std::time::Duration;
//use std::sync::{Arc, Mutex};
//use std::thread;
use tungstenite::{connect, Message};
//use url::Url;
use serde::{Deserialize, Serialize};
//use serde_json::{Value};

#[derive(Serialize, Deserialize)]
struct BidAsk {
    #[serde(rename = "type")]
    event_type: String,
    price: f64,
    amount: f64,
}

#[derive(Serialize, Deserialize)]
struct Data {
    #[serde(rename = "type")]
    event_type: String,
    timestamp: usize,
    prev_change_id: Option<usize>,
    instrument_name: String,
    change_id: usize,
    bids: Vec<BidAsk>,
    asks: Vec<BidAsk>,
}

#[derive(Serialize, Deserialize)]
struct Params {
    channel: String,
    data: Data,
}

#[derive(Serialize, Deserialize)]
struct Delta {
    jsonrpc: String,
    method: String,
    params: Params,
}

static REQUEST: &str = r#"
    {
        "method": "public/subscribe",
        "params": {
            "channels": [
                "book.BTC-PERPETUAL.100ms"
            ]
        },
        "jsonrpc": "2.0",
        "id": 0
    }"#;

fn main() {
    let (mut socket, _response) = connect("wss://test.deribit.com/ws/api/v2/").expect("Connection error");
    socket.write_message(Message::Text(REQUEST.to_string())).unwrap();

    loop {
        println!(" ");
        let msg = socket.read_message().expect("Reading error");
        if &Message::len(&msg) > &150 { // not interested in the handshake
            let parsed: Delta = serde_json::from_str(&Message::into_text(msg).unwrap()).unwrap();
            dbg!(&parsed.params.data.change_id);
        }
    }

}
