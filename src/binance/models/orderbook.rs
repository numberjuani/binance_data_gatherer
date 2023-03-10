use crate::binance::constants::Symbol;
use chrono::DateTime;
use chrono::Utc;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{serde_as, TimestampMilliSeconds};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
pub type OrderBooksRWL = Arc<RwLock<HashMap<String, Vec<OrderBook>>>>;
use rayon::prelude::*;

///`Arc::new(RwLock::new(HashMap::new()))`
pub fn new_orderbooks_rwl() -> OrderBooksRWL {
    Arc::new(RwLock::new(HashMap::new()))
}
use rust_decimal::Decimal;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct PriceSize {
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<PriceSize>,
    pub asks: Vec<PriceSize>,
    pub first_update_id: i64,
    pub last_update_id: i64,
    pub time: DateTime<Utc>,
    pub is_valid: bool,
}
impl OrderBook {
    pub fn new_from_update(update: OrderbookMessage) -> Self {
        Self {
            bids: update.bids,
            asks: update.asks,
            last_update_id: update.last_update_id,
            time: update.time,
            is_valid: true,
            first_update_id: update.first_update_id,
        }
    }
    pub fn update(mut self, update: OrderbookMessage) -> Self {
        //Check that the order of updates is whats expected, different process for spot and futures.
        let orderly = match update.prev_last_update_id {
            Some(previous) => previous == self.last_update_id,
            None => self.last_update_id == update.first_update_id - 1,
        };
        if !orderly {
            warn!("Orderbook update for {} not orderly",update.symbol);
            self.is_valid = false;
        }
        for bid in update.bids {
            if let Some(matching_bid) = self.bids.par_iter().position_any(|b| b.price == bid.price)
            {
                if bid.size.is_zero() {
                    self.bids.remove(matching_bid);
                } else {
                    self.bids[matching_bid].size = bid.size;
                }
            } else {
                self.bids.push(bid);
            }
        }
        for ask in update.asks {
            if let Some(matching_ask) = self.asks.par_iter().position_any(|a| a.price == ask.price)
            {
                if ask.size.is_zero() {
                    self.asks.remove(matching_ask);
                } else {
                    self.asks[matching_ask].size = ask.size;
                }
            } else {
                self.asks.push(ask);
            }
        }
        self.time = update.time;
        self.last_update_id = update.last_update_id;
        self.first_update_id = update.first_update_id;
        self.bids.par_sort_unstable_by_key(|b| -b.price);
        self.asks.par_sort_unstable_by_key(|a| a.price);
        self
    }
}
#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookMessage {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    #[serde_as(as = "TimestampMilliSeconds")]
    pub time: DateTime<Utc>,
    #[serde(rename = "s")]
    pub symbol: Symbol,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub last_update_id: i64,
    #[serde(rename = "b")]
    #[serde(with = "orderbook_serde")]
    pub bids: Vec<PriceSize>,
    #[serde(rename = "a")]
    #[serde(with = "orderbook_serde")]
    pub asks: Vec<PriceSize>,
    #[serde(rename = "pu")]
    pub prev_last_update_id: Option<i64>,
}
mod orderbook_serde {
    use rust_decimal::Decimal;
    use serde::{self, Deserialize, Deserializer};
    use std::str::FromStr;

    use super::PriceSize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PriceSize>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Vec<Vec<String>> = Vec::deserialize(deserializer)?;
        let mut v = Vec::new();
        for item in s {
            v.push(PriceSize {
                price: Decimal::from_str(&item[0]).unwrap(),
                size: Decimal::from_str(&item[1]).unwrap(),
            });
        }
        Ok(v)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCSVFormat {
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub quantity: Decimal,
}
