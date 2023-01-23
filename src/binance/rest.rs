use super::models::orderbook::OrderBook;
use super::models::orderbook::PriceSize;
use super::models::orderbook::UpdateCSVFormat;
use super::websocket::requests::DataRequest;
use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use log::error;
use log::info;
use log::warn;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Retrieves an orderbook snapshot from the REST API. If an endpoint fails, it will try the next one.
pub async fn get_orderbook(
    client: Client,
    symbol: &str,
    limit: u32,
    urls: Vec<String>,
) -> Result<RestOrderBook, reqwest::Error> {
    let mut tries = 0;
    let params = json!({
        "symbol": symbol,
        "limit": limit,
    });
    for url in &urls {
        tries += 1;
        match client.get(url).query(&params).send().await {
            Ok(response) => {
                return response.json::<RestOrderBook>().await;
            }
            Err(e) => {
                error!(
                    "Error getting orderbook from Binance: {:?} using {}",
                    e, url
                );
                if tries == urls.len() - 1 {
                    return Err(e);
                } else {
                    continue;
                }
            }
        }
    }
    Ok(RestOrderBook::empty())
}
/// Gets all the orderbooks at the same time from the REST API.
pub async fn get_orderbooks(
    request: DataRequest,
    limit: u32,
    snapshot_rwl: Arc<RwLock<HashMap<String, Vec<RestOrderBook>>>>,
    orderbooks_rwl: Arc<RwLock<HashMap<String, Vec<OrderBook>>>>,
) {
    let client = Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let symbols = request
        .streams
        .iter()
        .map(|s| s.get_symbol())
        .unique()
        .collect::<Vec<String>>();
    let urls = request.get_orderbook_endpoint();
    loop {
        interval.tick().await;
        info!("Getting orderbooks from Binance REST API");
        let futures = symbols
            .iter()
            .map(|symbol| get_orderbook(client.clone(), symbol, limit, urls.clone()));
        let orderbooks = futures::future::join_all(futures).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        let mut snapshot_write = snapshot_rwl.write().await;
        let books = orderbooks_rwl.read().await.clone();
        for (i, orderbook) in orderbooks.iter().enumerate() {
            match orderbook {
                Ok(snapshot) => {
                    match books.get(&symbols[i]) {
                        Some(books) => {
                            match books.par_iter().find_any(|book| {
                                book.first_update_id <= snapshot.last_update_id
                                    && book.last_update_id >= snapshot.last_update_id
                            }) {
                                Some(matching_book) => {
                                    if matching_book.last_update_id != snapshot.last_update_id {
                                        warn!("{} Loose comparison between update ids, in rest={}, in orderbook last update={}",symbols[i], snapshot.last_update_id, matching_book.last_update_id);
                                    }
                                    let mut levels_checked = 0;
                                    let mut errors = 0;
                                    for book_bid in &matching_book.bids {
                                        match snapshot.bids.par_iter().find_any(|snapshot_bid| {
                                            snapshot_bid.price == book_bid.price
                                        }) {
                                            Some(matching_bid) => {
                                                if matching_bid.size != book_bid.size && !matching_bid.size.is_zero() {
                                                    errors += 1;
                                                } else {
                                                    levels_checked += 1;
                                                }
                                            }
                                            None => {
                                                continue;
                                            }
                                        }
                                    }
                                    for ask in &matching_book.asks {
                                        match snapshot
                                            .asks
                                            .par_iter()
                                            .find_any(|a| a.price == ask.price)
                                        {
                                            Some(matching_ask) => {
                                                if matching_ask.size != ask.size && !matching_ask.size.is_zero() {
                                                    errors += 1;
                                                } else {
                                                    levels_checked += 1;
                                                }
                                            }
                                            None => {
                                                continue;
                                            }
                                        }
                                    }
                                    if errors == 0 {
                                        info!("Orderbook {} for {} is 100% accurate and up to date, levels checked {levels_checked}", snapshot.last_update_id, symbols[i]);
                                    } else {
                                        let accuracy = (levels_checked as f64 / (levels_checked + errors) as f64) * 100.0;
                                        warn!("Orderbook {} for {} is {:.2}% accurate, levels checked {levels_checked}, discrepancies {}", snapshot.last_update_id, symbols[i],accuracy, errors);
                                    }
                                }
                                None => {
                                    warn!(
                                        "An Orderbook encompassing {} for {} is not in the vec",
                                        snapshot.last_update_id, symbols[i]
                                    );
                                }
                            }
                        }
                        None => {
                            warn!("No orderbooks stored yet for {}", symbols[i]);
                        }
                    }
                    if !snapshot_write.contains_key(&symbols[i]) {
                        snapshot_write.insert(symbols[i].clone(), vec![snapshot.clone()]);
                    } else {
                        snapshot_write
                            .get_mut(&symbols[i])
                            .unwrap()
                            .push(snapshot.clone());
                    }
                }
                Err(e) => {
                    error!(
                        "Error getting orderbook from Binance: {:?} for {}",
                        e, symbols[i]
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize,Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RestOrderBook {
    pub last_update_id: i64,
    pub bids: Vec<PriceSize>,
    pub asks: Vec<PriceSize>,
    #[serde(default = "get_ts")]
    pub received_ts: DateTime<Utc>,
}
impl RestOrderBook {
    pub fn empty() -> Self {
        Self {
            last_update_id: 0,
            bids: Vec::new(),
            asks: Vec::new(),
            received_ts: Utc::now(),
        }
    }
    pub fn to_csv_format(&self) -> Vec<UpdateCSVFormat> {
        let mut output = Vec::new();
        for ask in &self.asks {
            output.push(UpdateCSVFormat {
                timestamp: self.received_ts,
                price: ask.price,
                quantity: -ask.size,
            });
        }
        for bid in &self.bids {
            output.push(UpdateCSVFormat {
                timestamp: self.received_ts,
                price: bid.price,
                quantity: bid.size,
            });
        }
        output
    }
}

pub fn get_ts() -> DateTime<Utc> {
    Utc::now()
}
