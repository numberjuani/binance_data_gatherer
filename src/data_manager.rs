use std::{sync::Arc, time::Duration, collections::HashMap};

use itertools::Itertools;
use log::{info, error};
use tokio::{sync::RwLock, time};

use crate::{binance::{models::{trades::Trade, orderbook::OrderbookMessage}, websocket::requests::DataRequest, rest::RestOrderBook}, file_compress::compress_file, bucket_utils::upload_object, settings::OUTGOING_FOLDER_NAME};

/// Creates files from the data received from the websocket and rest api. It also compresses and uploads the files to s3.
pub async fn create_files(update_messages: Arc<RwLock<Vec<OrderbookMessage>>>,trade_update_messages: Arc<RwLock<Vec<Trade>>>,request:DataRequest,snapshot_rwl: Arc<RwLock<HashMap<String, Vec<RestOrderBook>>>>) {
    tokio::time::sleep(Duration::from_secs(3600)).await;
    let mut interval = time::interval(Duration::from_secs(3600)); 
    loop {
        interval.tick().await;
        match std::fs::create_dir(OUTGOING_FOLDER_NAME) {
            Ok(_) => info!("Created folder {}", OUTGOING_FOLDER_NAME),
            Err(e) => error!("Error creating folder {}: {}", OUTGOING_FOLDER_NAME, e),
        }
        let mut update_messages = update_messages.write().await;
        let updates_copy = update_messages.clone();
        update_messages.clear();
        drop(update_messages);
        let update_symbols = updates_copy.iter().map(|x| x.symbol.clone()).unique().collect_vec();
        for symbol in update_symbols {
            let mut rows = Vec::new();
            for update in &updates_copy {
                if update.symbol == symbol {
                    rows.extend(update.to_csv_format())
                }
            }
            let symbol = format!("{}/{}_{}_BOOK_HISTORY",OUTGOING_FOLDER_NAME,request.asset_type,symbol);
            let filename = format!("{}.csv",symbol);
            create_csv_file(&rows, &filename);
        }
        let snapshots = snapshot_rwl.write().await;
        let snapshots_copy = snapshots.clone();
        drop(snapshots);
        for (key,value) in snapshots_copy.iter() {
            let mut rows = Vec::new();
            for snapshot in value {
                rows.extend(snapshot.to_csv_format())
            }
            let filename = format!("{}/{}_{}_BOOK_SNAPSHOT.csv",OUTGOING_FOLDER_NAME,request.asset_type,key);
            create_csv_file(&rows, &filename);
        }
        let mut trades = trade_update_messages.write().await;
        let trades_copy = trades.clone();
        trades.clear();
        drop(trades);
        create_csv_file(&trades_copy, format!("{}/TRADES.csv",OUTGOING_FOLDER_NAME).as_str());
        //list the contents of the outgoing directory
        let files = std::fs::read_dir(OUTGOING_FOLDER_NAME).unwrap();
        for file in files {
            match file {
                Ok(file) => {
                    let file_name = format!("{OUTGOING_FOLDER_NAME}/{}",file.file_name().into_string().unwrap());
                    match compress_file(&file_name) {
                        Ok(compressed_filename) => {
                            match upload_object("buckent_name:m",&compressed_filename,"key").await {
                                Ok(_) => {
                                    info!("File {} uploaded to s3",compressed_filename);
                                    std::fs::remove_file(&compressed_filename).unwrap();
                                },
                                Err(e) => error!("Error uploading file to s3: {}",e)
                            }
                        },
                        Err(e) => {
                            error!("Error compressing file: {}",e);
                        }
                    }
                },
                Err(e) => {
                    error!("Error reading file: {}",e);
                }
            }
        }
    }
}


pub fn create_csv_file<T: serde::Serialize>(data: &[T], filename: &str) {
    if !data.is_empty() {
        match csv::Writer::from_path(filename) {
            Ok(mut writer) => {
                for line in data {
                    writer.serialize(line).unwrap();
                }
                info!("Succesfully Created file {}", filename);
            },
            Err(e) => {
                log::error!("Error creating csv file: {} {}", e, filename);
            },
        }
    }
}