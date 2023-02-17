use chrono::DateTime;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{serde_as, TimestampMilliSeconds};
use chrono::Utc;
use rust_decimal::Decimal;
#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Trade {
    #[serde(rename(deserialize = "e"))]
    pub event_type: String,
    #[serde(rename(deserialize = "E"))]
    #[serde_as(as = "TimestampMilliSeconds")]
    pub event_time: DateTime<Utc>,
    #[serde(rename(deserialize = "T"))]
    #[serde_as(as = "TimestampMilliSeconds")]
    pub trade_time: DateTime<Utc>,
    #[serde(rename(deserialize = "s"))]
    pub symbol: String,
    #[serde(rename(deserialize = "t"))]
    pub trade_id: i64,
    #[serde(rename(deserialize = "p"))]
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(rename(deserialize = "q"))]
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,
    #[serde(rename(deserialize = "X"))]
    pub x: Option<String>,
    #[serde(rename(deserialize = "m"))]
    pub buyer_is_the_market_maker: bool,
}
impl Trade {
    pub fn side(&self) -> String {
        if self.buyer_is_the_market_maker {
            "SELL".to_string()
        } else {
            "BUY".to_string()
        }
    }
    pub fn get_data(&self) {
        println!("Trade: {:?} {} {}", self,self.side(),self.price*self.quantity);
    }
}