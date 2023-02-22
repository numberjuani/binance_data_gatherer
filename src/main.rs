use crate::binance::{
    models::orderbook::new_orderbooks_rwl,
    websocket::requests::{BinanceAssetType, DataRequest, FuturesType, Stream},
};

mod binance;

pub const ALL_SYMBOLS: [&str; 206] = [
    "RUNEUSDT",
    "GMXUSDT",
    "BTCBUSD",
    "AGIXUSDT",
    "APTBUSD",
    "TRXUSDT",
    "DOGEUSDT",
    "IMXUSDT",
    "BNBUSDT",
    "RAYUSDT",
    "NKNUSDT",
    "OCEANUSDT",
    "JASMYUSDT",
    "MATICBUSD",
    "GMTUSDT",
    "LDOUSDT",
    "MANAUSDT",
    "API3USDT",
    "STGUSDT",
    "ADABUSD",
    "1000XECUSDT",
    "KLAYUSDT",
    "1000SHIBBUSD",
    "CTKUSDT",
    "ARPAUSDT",
    "XRPUSDT",
    "IOTXUSDT",
    "DYDXUSDT",
    "TOMOUSDT",
    "HBARUSDT",
    "CELRUSDT",
    "FXSUSDT",
    "ALPHAUSDT",
    "NEOUSDT",
    "ETCBUSD",
    "SOLBUSD",
    "CVCUSDT",
    "ZILUSDT",
    "RENUSDT",
    "PHBUSDT",
    "FLOWUSDT",
    "ZENUSDT",
    "DODOBUSD",
    "SANDBUSD",
    "CTSIUSDT",
    "GALBUSD",
    "AUCTIONBUSD",
    "AVAXUSDT",
    "CVXUSDT",
    "VETUSDT",
    "MTLUSDT",
    "XEMUSDT",
    "ANCBUSD",
    "TUSDT",
    "TRBUSDT",
    "FTTBUSD",
    "DOTUSDT",
    "LTCBUSD",
    "ALICEUSDT",
    "INJUSDT",
    "LDOBUSD",
    "NEARBUSD",
    "EGLDUSDT",
    "LPTUSDT",
    "CRVUSDT",
    "ONEUSDT",
    "FILUSDT",
    "EOSUSDT",
    "SXPUSDT",
    "ANTUSDT",
    "HIGHUSDT",
    "FTMUSDT",
    "MAGICUSDT",
    "HNTUSDT",
    "HOTUSDT",
    "BTCDOMUSDT",
    "ETHBUSD",
    "RLCUSDT",
    "UNIUSDT",
    "SOLUSDT",
    "ICXUSDT",
    "MINAUSDT",
    "SKLUSDT",
    "SUSHIUSDT",
    "TRXBUSD",
    "SPELLUSDT",
    "ENSUSDT",
    "ATOMUSDT",
    "DGBUSDT",
    "APEUSDT",
    "SCUSDT",
    "GALUSDT",
    "BATUSDT",
    "COMPUSDT",
    "XRPBUSD",
    "DUSKUSDT",
    "MATICUSDT",
    "AMBBUSD",
    "IOTAUSDT",
    "CVXBUSD",
    "LUNA2USDT",
    "FETUSDT",
    "BTSUSDT",
    "KAVAUSDT",
    "AGIXBUSD",
    "BALUSDT",
    "MASKUSDT",
    "FLMUSDT",
    "ENJUSDT",
    "ATAUSDT",
    "OPUSDT",
    "BANDUSDT",
    "WOOUSDT",
    "PEOPLEUSDT",
    "OMGUSDT",
    "DENTUSDT",
    "ARUSDT",
    "1000LUNCBUSD",
    "ETCUSDT",
    "ICPBUSD",
    "LUNA2BUSD",
    "1000SHIBUSDT",
    "SRMUSDT",
    "LRCUSDT",
    "ZRXUSDT",
    "LINKUSDT",
    "UNIBUSD",
    "LITUSDT",
    "CHRUSDT",
    "FTTUSDT",
    "COTIUSDT",
    "1000LUNCUSDT",
    "CELOUSDT",
    "WAVESUSDT",
    "GALAUSDT",
    "OGNUSDT",
    "YFIUSDT",
    "AVAXBUSD",
    "BAKEUSDT",
    "MKRUSDT",
    "RVNUSDT",
    "DARUSDT",
    "BELUSDT",
    "NEARUSDT",
    "DEFIUSDT",
    "RSRUSDT",
    "QNTUSDT",
    "AXSUSDT",
    "BNBBUSD",
    "ETHUSDT",
    "XLMUSDT",
    "UNFIUSDT",
    "DOGEBUSD",
    "KSMUSDT",
    "ONTUSDT",
    "BLZUSDT",
    "DOTBUSD",
    "SNXUSDT",
    "WAVESBUSD",
    "HOOKUSDT",
    "KNCUSDT",
    "ZECUSDT",
    "ALGOUSDT",
    "C98USDT",
    "ROSEUSDT",
    "CHZUSDT",
    "ASTRUSDT",
    "SANDUSDT",
    "LTCUSDT",
    "STORJUSDT",
    "APTUSDT",
    "PHBBUSD",
    "GMTBUSD",
    "FILBUSD",
    "BLUEBIRDUSDT",
    "THETAUSDT",
    "LINKBUSD",
    "GRTUSDT",
    "REEFUSDT",
    "ANKRUSDT",
    "AUDIOUSDT",
    "GTCUSDT",
    "LINAUSDT",
    "BNXUSDT",
    "FTMBUSD",
    "TLMBUSD",
    "BTCUSDT",
    "1INCHUSDT",
    "RNDRUSDT",
    "STMXUSDT",
    "LEVERBUSD",
    "AAVEUSDT",
    "SFPUSDT",
    "FOOTBALLUSDT",
    "APEBUSD",
    "XTZUSDT",
    "DASHUSDT",
    "GALABUSD",
    "TLMUSDT",
    "IOSTUSDT",
    "XMRUSDT",
    "ICPUSDT",
    "BTCSTUSDT",
    "ADAUSDT",
    "QTUMUSDT",
    "BCHUSDT",
];

#[tokio::main]
async fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let request = DataRequest::new(
        BinanceAssetType::Futures(FuturesType::USDMargined),
        ALL_SYMBOLS
            .iter()
            .map(|s| Stream::Trade(s.to_string()))
            .collect(),
    );
    let orderbooks_rwl = new_orderbooks_rwl();
    let trade_update_messages = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    _ = tokio::join!(tokio::spawn(
        binance::websocket::connection::establish_and_persist(
            request.clone(),
            orderbooks_rwl.clone(),
            trade_update_messages.clone()
        )
    ),);
}
