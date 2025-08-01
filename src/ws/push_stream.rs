use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{
        Query,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{RwLock, broadcast},
};
use tokio_tungstenite::{accept_hdr_async, tungstenite::Message};

use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

use crate::data::coin_market::CoinsMarket;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SymbolConfig {
    symbol: String,
    interval_ms: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct MarketPriceData {
    symbol: String,
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Deserialize, Debug)]
pub struct WsParams {
    exchange: Option<String>,
    symbol: Option<String>,
    interval_ms: Option<u64>,
}

pub type BroadcastMap = Arc<RwLock<HashMap<(String, u64), broadcast::Sender<MarketPriceData>>>>;

pub async fn start_ws_server(addr: String) {
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("WebSocket server listening on ws://{}", addr);

    let broadcast_map: BroadcastMap = Arc::new(RwLock::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let broadcast_map = broadcast_map.clone();
        tokio::spawn(async move {
            handle_connection(stream, broadcast_map).await;
        });
    }
}

async fn handle_connection(stream: TcpStream, broadcast_map: BroadcastMap) {
    let mut symbol = "BTCUSDT".to_string();
    let mut interval_ms = 1000u64;

    let callback = |req: &Request, mut response: Response| {
        if let Some(path_and_query) = req.uri().path_and_query() {
            if let Ok(parsed_url) = Url::parse(&format!("ws://localhost{}", path_and_query)) {
                let query_map = parsed_url
                    .query_pairs()
                    .into_owned()
                    .collect::<HashMap<_, _>>();
                if let Some(s) = query_map.get("symbol") {
                    symbol = s.to_string();
                }
                if let Some(i) = query_map.get("interval_ms") {
                    if let Ok(v) = i.parse::<u64>() {
                        interval_ms = v;
                    }
                }
            }
        }
        Ok(response)
    };

    let ws_stream = accept_hdr_async(stream, callback)
        .await
        .expect("Failed to accept websocket");

    println!("New WebSocket connection");

    println!("symbol: {}, interval_ms: {}", symbol, interval_ms);

    let key = (symbol.clone(), interval_ms);
    let (mut write, mut read) = ws_stream.split();

    // if let Some(Ok(msg)) = read.next().await {
    let sender = {
        let mut map = broadcast_map.write().await;

        map.entry(key.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                tokio::spawn(start_price_task(
                    symbol.to_string(),
                    interval_ms,
                    tx.clone(),
                ));
                tx
            })
            .clone()
    };

    let mut rx: broadcast::Receiver<MarketPriceData> = sender.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(data) => {
                        let json = serde_json::to_string(&data).unwrap();
                        if write.send(Message::Text(json.into())).await.is_err() {
                            println!("Client disconnected (write error)");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        println!("Client lagged behind: skipped {} messages", skipped);
                    }
                    Err(_) => break,
                }
            }

            client_msg = read.next() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) => {
                        println!("Client sent close frame");
                        let _ = write.send(Message::Close(None)).await;
                        break;
                    }
                    Some(Ok(_)) => {
                    }
                    Some(Err(e)) => {
                        println!("WebSocket read error: {}", e);
                        break;
                    }
                    None => {
                        println!("WebSocket stream ended");
                        break;
                    }
                }
            }
        }
    }
    // }
}

async fn start_price_task(
    symbol: String,
    interval_ms: u64,
    tx: broadcast::Sender<MarketPriceData>,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
    loop {
        interval.tick().await;

        let price_data = MarketPriceData {
            symbol: symbol.to_string(),
            timestamp: current_ts(),
            open: rand_price_by_symbol(&symbol),
            high: rand_price_by_symbol(&symbol),
            low: rand_price_by_symbol(&symbol),
            close: rand_price_by_symbol(&symbol),
            volume: rand_price_by_symbol(&symbol),
        };

        let _ = tx.send(price_data);
    }
}

fn rand_price() -> f64 {
    10000.0 + rand::random::<f64>() * 1000.0
}

fn rand_price_by_symbol(symbol: &str) -> f64 {
    let mut rng = rand::thread_rng();
    let (base, delta) = match symbol {
        "BTC/USDT" => (10500.0, 300.0),
        "ETH/USDT" => (3500.0, 100.0),
        "SOL/USDT" => (110.0, 15.0),
        _ => (10000.0, 500.0),
    };

    return base + rng.gen_range(-delta..delta);
}

fn current_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub async fn handle_web_socket(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    broadcast_map: axum::extract::Extension<BroadcastMap>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| client_ws(socket, params, broadcast_map.0))
}

async fn client_ws(mut socket: WebSocket, params: WsParams, broadcast_map: BroadcastMap) {
    let symbol = params.symbol.unwrap_or_else(|| "BTC/USDT".to_string());
    let interval_ms = params.interval_ms.unwrap_or(1000);
    let key = (symbol.clone(), interval_ms);

    let sender = {
        let mut map = broadcast_map.write().await;
        map.entry(key.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                tokio::spawn(start_price_task(symbol.clone(), interval_ms, tx.clone()));
                tx
            })
            .clone()
    };
    let mut rx = sender.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(data) => {
                        let json = serde_json::to_string(&data).unwrap();
                        if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {},
                    Err(_) => break,
                }
            }
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
