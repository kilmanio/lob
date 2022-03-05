//use std::time::Duration;
//use std::sync::{Arc, Mutex};
//use std::thread;
use tungstenite::{connect, Message};
//use url::Url;
use serde::{Deserialize, Serialize};
//use serde_json::{Value};
use std::collections::HashMap;
use std::process::exit;
//use fixed::FixedU64;
//use fixed::types::extra::U32;
use rust_decimal::prelude::*;

#[derive(Serialize, Deserialize)]
struct BidAsk {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(with = "rust_decimal::serde::float")]
    price: Decimal,
    amount: f64,
}

#[derive(Serialize, Deserialize)]
struct Data {
    #[serde(rename = "type")]
    event_type: String,
    timestamp: usize,
    prev_change_id: Option<usize>, // the snapshot doesn't have this
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

fn change(ba: BidAsk, map: &mut HashMap<Decimal, f64>) {
    map.insert(ba.price, ba.amount);
}

fn delete(ba: BidAsk, map: &mut HashMap<Decimal, f64>) {
    map.remove(&ba.price);
}

//same as change
fn new(ba: BidAsk, map: &mut HashMap<Decimal, f64>) {
    map.insert(ba.price, ba.amount);
}

fn handle_bid_ask(ba: BidAsk, map: &mut HashMap<Decimal, f64>){
    match ba.event_type.as_str() {
        "change" => change(ba, map),
        "delete" => delete(ba, map),
        "new" => new(ba, map),
        _ => exit(1),
    }
}

fn handle(d: Delta, bids: &mut HashMap<Decimal, f64>, asks: &mut HashMap<Decimal, f64>) {
    for bid in d.params.data.bids {
        handle_bid_ask(bid, bids);
    }
    for ask in d.params.data.asks {
        handle_bid_ask(ask, asks);
    }
}

fn main() {
    let mut bids: HashMap<Decimal, f64> = HashMap::new();
    let mut asks: HashMap<Decimal, f64> = HashMap::new();

    let (mut socket, _response) = connect("wss://test.deribit.com/ws/api/v2/").expect("Connection error");
    socket.write_message(Message::Text(REQUEST.to_string())).unwrap();

    loop {
        println!("");
        let msg = socket.read_message().expect("Reading error");
        if &Message::len(&msg) > &150 { // arbitrary number, not interested in the handshake
            let parsed: Delta = serde_json::from_str(&Message::into_text(msg).unwrap()).unwrap();
            handle(parsed, &mut bids, &mut asks);
            //dbg!(&parsed.params.data.change_id);
        }
        //dbg!(&bids);
    }

}
