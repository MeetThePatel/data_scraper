use chrono::prelude::*;
use chrono::Duration;
use futures_util::StreamExt;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_postgres::NoTls;

#[derive(Debug, Serialize, Deserialize)]
struct CsvRow {
    open_time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    close_time: u64,
    quote_asset_volume: f64,
    number_of_trades: i64,
    taker_buy_base_asset_volume: f64,
    taker_buy_quote_asset_volume: f64,
    ignore: u8,
}

async fn binance_get_csv(symbol: &str, date: &Date<Utc>) -> Vec<u8> {
    let url = std::format!(
        "https://data.binance.vision/data/spot/daily/klines/{}/1m/{}-1m-{}.zip",
        symbol,
        symbol,
        &date.format("%Y-%m-%d").to_string()
    );
    let response = reqwest::get(url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();
    let mut zip_reader = async_zip::read::mem::ZipFileReader::new(&response)
        .await
        .unwrap();
    zip_reader
        .entry_reader(0)
        .await
        .unwrap()
        .read_to_end_crc()
        .await
        .unwrap()
}

async fn read_csv_into_postgres(
    csv_bytes: &[u8],
    symbol: &str,
    client: &tokio_postgres::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = csv_async::AsyncReaderBuilder::new()
        .delimiter(b',')
        .create_reader(csv_bytes);
    let mut records = reader.into_records();

    while let Some(Ok(record)) = records.next().await {
        let row: CsvRow = record.deserialize(None)?;
        let _date: DateTime<Utc> =
            DateTime::from_utc(NaiveDateTime::from_timestamp(row.open_time / 1000, 0), Utc);
        client
            .execute(
                "INSERT INTO kline_1m VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &_date,
                    &symbol,
                    &row.open,
                    &row.high,
                    &row.low,
                    &row.close,
                    &row.volume,
                    &row.quote_asset_volume,
                    &row.number_of_trades,
                    &row.taker_buy_base_asset_volume,
                    &row.taker_buy_quote_asset_volume,
                ],
            )
            .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let (tx, mut rx) = mpsc::channel(1000);

    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres dbname=binance", NoTls).await?;

    let _postgres_thread = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection Error: {}", e);
        }
    });

    let start_date = Utc.ymd(2021, 03, 01);
    let end_date = Utc.ymd(2022, 06, 10);

    let mut date_iter = start_date;
    while date_iter <= end_date {
        let tx_cp = tx.clone();

        let pair = args[1].clone() + &args[2];
        tokio::spawn(async move {
            tx_cp
                .send((binance_get_csv(&pair, &date_iter).await, date_iter.clone()))
                .await
                .unwrap();
        });
        date_iter = date_iter + Duration::days(1);
    }

    while let Some((csv_bytes, date)) = rx.recv().await {
        read_csv_into_postgres(&csv_bytes, &(args[1].clone() + " " + &args[2]), &client).await?;
        println!("{}", date);
    }

    Ok(())
}
