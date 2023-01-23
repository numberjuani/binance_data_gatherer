
use chrono::Duration;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, error, info, warn};
use serde_json::{Map, Value};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::binance::{models::{orderbook::{OrderBooksRWL, OrderbookMessage}, trades::Trade}, websocket::handlers::book_ticker::handle_book_ticker};

use super::{requests::DataRequest, handlers::{depth_update::handle_depth_update_message, trades::handle_trades}};

type OutgoingSocket = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type IncomingSocket = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
/// Establishes a websocket connection to Binance and persists it for the duration of the program.
/// If disconnected, it will attempt to reconnect uo to 5 times at an ever-increasing interval up to 26 seconds.
/// It will try a max of 5 times before exiting the program.
pub async fn establish_and_persist(requests: DataRequest, orderbooks_rwl:OrderBooksRWL,update_messages_rwl:Arc<RwLock<Vec<OrderbookMessage>>>,trade_updates_rwl:Arc<RwLock<Vec<Trade>>>) {
    let mut bad_attempts = 0;
    loop {
        if establish(requests.clone(),orderbooks_rwl.clone(),update_messages_rwl.clone(),trade_updates_rwl.clone()).await {
            bad_attempts += 1;
        } else {
            bad_attempts = 0;
        }
        if bad_attempts >= 5 {
            warn!("Too many failed attempts to connect to websocket, exiting");
            return;
        }
        tokio::time::sleep(Duration::seconds(bad_attempts * 5 + 1).to_std().unwrap()).await;
    }
}
/// Establishes a single websocket connection to Binance. Returns true if there was an error.
async fn establish(request: DataRequest,orderbooks_rwl:OrderBooksRWL,update_messages_rwl:Arc<RwLock<Vec<OrderbookMessage>>>,trade_updates_rwl:Arc<RwLock<Vec<Trade>>>) -> bool {
    for endpoint in request.get_ws_urls().iter() {
        info!("Attempting WS connection to {}", endpoint);
        match tokio_tungstenite::connect_async(endpoint).await {
            Ok((stream, _)) => {
                info!("Connected to {endpoint}");
                let (sender, receiver) = stream.split();
                let close = Arc::new(tokio::sync::RwLock::new(false));
                let ping_pong = Arc::new(tokio::sync::RwLock::new(false));
                match tokio::try_join!(
                    tokio::spawn(process_incoming_message(
                        receiver,
                        close.clone(),
                        ping_pong.clone(),
                        orderbooks_rwl.clone(),
                        update_messages_rwl.clone(),
                        trade_updates_rwl.clone()
                    )),
                    tokio::spawn(process_outgoing_message(
                        sender,
                        close,
                        ping_pong,
                        request.clone()
                    )),
                ) {
                    Ok(_) => {
                        info!("Connection closed");
                        return false;
                    }
                    Err(e) => {
                        error!("Disconnected from Binance websocket: {:?}", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e);
                continue;
            }
        }
    }
    true
}

async fn process_incoming_message(
    receiver: IncomingSocket,
    close: Arc<tokio::sync::RwLock<bool>>,
    ping_pong: Arc<tokio::sync::RwLock<bool>>,
    orderbooks_rwl:OrderBooksRWL,
    update_messages_rwl:Arc<RwLock<Vec<OrderbookMessage>>>,
    trade_updates_rwl:Arc<RwLock<Vec<Trade>>>
) {
    receiver
        .for_each(|message| async {
            if *close.read().await {
                return;
            }
            match message {
                Ok(text_message) => match text_message {
                    Message::Text(text_message) => {
                        debug!("Received message: {}", text_message);
                        match serde_json::from_str::<Map<String, Value>>(&text_message) {
                            Ok(unrouted_message) => match unrouted_message.contains_key("data") {
                                true => match unrouted_message["data"]["e"].as_str().unwrap() {
                                    "depthUpdate" => {
                                        handle_depth_update_message(unrouted_message["data"].clone(), orderbooks_rwl.clone(),update_messages_rwl.clone()).await;
                                    }
                                    "trade" => {
                                        handle_trades(unrouted_message["data"].clone(), trade_updates_rwl.clone()).await;
                                    }
                                    "bookTicker" => {
                                        handle_book_ticker(unrouted_message["data"].clone()).await;
                                    }
                                    _ => {
                                        debug!("Unrecognized message: {:?}", unrouted_message);
                                    }
                                },
                                false => {
                                    if unrouted_message.keys().len() == 2
                                        && unrouted_message.contains_key("result")
                                        && unrouted_message["result"].is_null()
                                    {
                                        info!(
                                            "Successfully subscribed to request id {}",
                                            unrouted_message["id"]
                                        );
                                    } else {
                                        warn!("Unrecognized message: {:?}", unrouted_message);
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Error parsing message: {:?}", e);
                            }
                        }
                    }
                    Message::Binary(_) => {
                        warn!("Binary message received");
                    }
                    Message::Ping(_) => {
                        info!("Received ping");
                        let mut ping_pong = ping_pong.write().await;
                        *ping_pong = true;
                    }
                    Message::Pong(_) => {}
                    Message::Close(cf) => {
                        warn!("Close received {cf:?}");
                        let mut close = close.write().await;
                        *close = true;
                    }
                    Message::Frame(_) => {
                        warn!("Frame received");
                    }
                },
                Err(e) => {
                    warn!("Error receiving message: {:?}", e);
                    let mut close = close.write().await;
                    *close = true;
                }
            }
        })
        .await;
}

async fn process_outgoing_message(
    mut sender: OutgoingSocket,
    close: Arc<tokio::sync::RwLock<bool>>,
    ping_pong: Arc<tokio::sync::RwLock<bool>>,
    request: DataRequest,
) {
    let sub_message = request.get_subscribe_message();
    match sender.send(Message::Text(sub_message.clone())).await {
        Ok(_) => {
            info!("Sent message {}", sub_message.clone());
        }
        Err(e) => {
            error!("Error {:?} sending {}", e, sub_message);
        }
    }
    loop {
        if *close.read().await {
            break;
        }
        if *ping_pong.read().await {
            match sender.send(Message::Pong(vec![])).await {
                Ok(_) => {
                    info!("Sent pong");
                    let mut ping_pong = ping_pong.write().await;
                    *ping_pong = false;
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
}
