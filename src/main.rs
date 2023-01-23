use std::collections::HashMap;
mod data_manager;
use crate::binance::{
    models::orderbook::new_orderbooks_rwl,
    websocket::requests::{BinanceAssetType, DataRequest, FuturesType, Stream},
};
mod binance;
mod bucket_utils;
mod file_compress;
mod settings;


#[tokio::main]
async fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let request = DataRequest::new(
        BinanceAssetType::Futures(FuturesType::USDMargined),
        vec![Stream::Trade("BTCUSDT".to_string())],
    );
    let orderbooks_rwl = new_orderbooks_rwl();
    let book_update_messages = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let trade_update_messages = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let snapshot_rwl = std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    tokio::join!(
        binance::websocket::connection::establish_and_persist(
            request.clone(),
            orderbooks_rwl.clone(),
            book_update_messages.clone(),
            trade_update_messages.clone()
        ),
        binance::rest::get_orderbooks(
            request.clone(),
            1000,
            snapshot_rwl.clone(),
            orderbooks_rwl.clone()
        ),
        data_manager::create_files(
            book_update_messages.clone(),
            trade_update_messages.clone(),
            request.clone(),
            snapshot_rwl.clone()
        )
    );
}
