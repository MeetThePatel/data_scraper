#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct HistDataResponseUnit {
    startTime: chrono::DateTime<chrono::Utc>,
    time: f64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct HistDataResponse {
    success: bool,
    result: Vec<HistDataResponseUnit>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=crypto",
        tokio_postgres::NoTls,
    )
    .await?;

    let _postgres_thread = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection Error: {}", e);
        }
    });

    let args: Vec<String> = std::env::args().collect();

    let base_curr: String = args[1].to_owned();
    let quote_curr: String = args[2].to_owned();

    let pair = format!("{} {}", base_curr, quote_curr);
    let start_datetime = chrono::NaiveDate::from_ymd(2020, 01, 01).and_hms(0, 0, 0);
    let end_datetime = chrono::NaiveDate::from_ymd(2022, 06, 16).and_hms(0, 0, 0);

    let mut dt = start_datetime;
    while dt <= end_datetime {
        let body = reqwest::get(format!(
            "https://ftx.com/api/markets/{}/{}/candles?resolution=15&start_time={}&end_time={}",
            base_curr,
            quote_curr,
            dt.timestamp(),
            (dt + chrono::Duration::hours(4) - chrono::Duration::seconds(15)).timestamp()
        ))
        .await?
        .text()
        .await?;
        let r: HistDataResponse = serde_json::from_str(&body).unwrap();
        for res in r.result {
            client
                .execute(
                    "INSERT INTO ftx_kline_15s VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    &[
                        &res.startTime,
                        &pair,
                        &res.open,
                        &res.high,
                        &res.low,
                        &res.close,
                        &res.volume,
                    ],
                )
                .await?;
        }
        println!("Finished {} - {}", dt, dt + chrono::Duration::hours(4) - chrono::Duration::seconds(15));
        dt = dt + chrono::Duration::hours(4);
    }

    Ok(())
}
