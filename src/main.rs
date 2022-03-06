use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use tungstenite::{connect, Message};

use std::sync::mpsc;

#[derive(Serialize, Deserialize)]
struct BidAsk {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(with = "rust_decimal::serde::float")]
    price: Decimal,
    amount: f64, //CHANGE TO USIZE
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

fn change(ba: &BidAsk, map: &mut HashMap<Decimal, f64>) {
    map.insert(ba.price, ba.amount);
}

fn delete(ba: &BidAsk, map: &mut HashMap<Decimal, f64>) {
    map.remove(&ba.price);
}

fn handle_bid_ask(ba: &BidAsk, map: &mut HashMap<Decimal, f64>) {
    match ba.event_type.as_str() {
        "change" => change(ba, map),
        "delete" => delete(ba, map),
        "new" => change(ba, map),
        _ => panic!("API error"),
    }
}

fn handle(d: &Delta, bids: &mut HashMap<Decimal, f64>, asks: &mut HashMap<Decimal, f64>) {
    for bid in &d.params.data.bids {
        handle_bid_ask(bid, bids);
    }
    for ask in &d.params.data.asks {
        handle_bid_ask(ask, asks);
    }
}

fn get_highest(map: &HashMap<Decimal, f64>) -> Decimal {
    *map.keys()
        .cloned()
        .collect::<Vec<Decimal>>()
        .iter()
        .max()
        .unwrap()
}

fn get_lowest(map: &HashMap<Decimal, f64>) -> Decimal {
    *map.keys()
        .cloned()
        .collect::<Vec<Decimal>>()
        .iter()
        .min()
        .unwrap()
}

fn main() {
    let mut bids: HashMap<Decimal, f64> = HashMap::new();
    let mut asks: HashMap<Decimal, f64> = HashMap::new();
    bids.insert(Decimal::new(0, 5), 1.0); //DEBUGGING
    asks.insert(Decimal::new(9999999, 0), 1.0); //DEBUGGING
    
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let line = rx.recv().unwrap();
            println!("{}",line);
        }
    });

    let (mut socket, _response) =
        connect("wss://test.deribit.com/ws/api/v2/").expect("Connection error");
    socket
        .write_message(Message::Text(REQUEST.to_string()))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Reading error");
        // 150 is high enough to not catch the handshake
        if &Message::len(&msg) > &150 {
            let parsed: Delta = serde_json::from_str(&Message::into_text(msg).unwrap()).unwrap();
            handle(&parsed, &mut bids, &mut asks);
            let best_bid = get_highest(&bids);
            let best_ask = get_lowest(&asks);
            let line = format!("Best bid price: {0}, count: {1}. Best ask price: {2}, count: {3}",
                    best_bid, bids[&best_bid], best_ask, asks[&best_ask]
                    );
            tx.send(line).unwrap();
        }
    }
}
