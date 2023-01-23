#[cfg(test)]

#[tokio::test]
async fn test_url_builder() {
    let urls = crate::binance::websocket::requests::DataRequest::new(
        crate::binance::websocket::requests::BinanceAssetType::Spot,
        vec![crate::binance::websocket::requests::Stream::Depth("BTCUSDT".to_string(), 1000), 
            crate::binance::websocket::requests::Stream::Depth("ETHUSDT".to_string(), 1000)
        ],
    ).get_ws_urls();
    assert!(urls[0] == "wss://stream.binance.com:9443/stream?streams=btcusdt@depth@1000ms/ethusdt@depth@1000ms");
}