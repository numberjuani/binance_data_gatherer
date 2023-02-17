use crate::binance::{
    models::orderbook::new_orderbooks_rwl,
    websocket::requests::{BinanceAssetType, DataRequest, FuturesType, Stream},
};

mod binance;

#[tokio::main]
async fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let request = DataRequest::new(
        BinanceAssetType::Futures(FuturesType::USDMargined),
        vec![
            Stream::Trade("BTCUSDT".to_string()),
            Stream::Trade("ETHUSDT".to_string()),
            Stream::Trade("SOLUSDT".to_string()),
            Stream::Trade("BNBUSDT".to_string()),
        ],
    );
    let orderbooks_rwl = new_orderbooks_rwl();
    let trade_update_messages = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    _ = tokio::join!(
        tokio::spawn(binance::websocket::connection::establish_and_persist(
            request.clone(),
            orderbooks_rwl.clone(),
            trade_update_messages.clone()
        )),
    );
}
