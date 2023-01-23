use std::sync::Arc;
use log::error;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use tokio::sync::RwLock;
use crate::binance::models::orderbook::{OrderBooksRWL, OrderbookMessage, OrderBook};

pub async fn handle_depth_update_message(message: Value, orderbooks_rwl: OrderBooksRWL,update_messages_rwl:Arc<RwLock<Vec<OrderbookMessage>>>) {
    match serde_json::from_value::<OrderbookMessage>(message) {
        Ok(update) => {
            update_messages_rwl.write().await.push(update.clone());
            let books = orderbooks_rwl.read().await.clone();
            match books.contains_key(&update.symbol) {
                true => {
                    let mut all_books = books.get(&update.symbol).unwrap().clone();
                    let latest_updated = all_books.par_iter().max_by_key(|book| book.last_update_id).unwrap().clone().update(update.clone());
                    all_books.insert(0,latest_updated.clone());
                    all_books.truncate(100);
                    orderbooks_rwl.write().await.insert(update.symbol.clone(), all_books);
                },
                false => {
                    let mut vec = Vec::with_capacity(100);
                    let book = OrderBook::new_from_update(update.clone());
                    vec.push(book);
                    orderbooks_rwl.write().await.insert(update.symbol.clone(),vec );
                },
            }
            
        }
        Err(e) => {
            error!("Error parsing message: {:?}", e);
        }
    }
}
