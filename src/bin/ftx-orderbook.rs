use chrono::{SubsecRound, Timelike};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FTXOrderBookLevel {
    price: f64,
    size: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FTXOrderBookResponse {
    bids: Vec<FTXOrderBookLevel>,
    asks: Vec<FTXOrderBookLevel>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FTXOrderBookResponseWrapper {
    success: bool,
    result: FTXOrderBookResponse,
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
            eprintln!("Connection  Error: {}", e);
        }
    });

    let args: Vec<String> = std::env::args().collect();

    loop {
        if chrono::Utc::now().second() % 15 == 0 {
            let time = chrono::Utc::now().round_subsecs(0);

            for pair in &args[1..] {
                let p: Vec<&str> = pair.split(' ').collect();
                let base_curr = p[0];
                let quote_curr = p[1];

                let url = format!(
                    "https://ftx.com/api/markets/{}/{}/orderbook?depth=20",
                    base_curr, quote_curr
                );

                let body = reqwest::get(url).await?.text().await?;
                let mut resp: FTXOrderBookResponseWrapper = serde_json::from_str(&body)?;
                resp.result.bids.reverse();

                // Ingest bids.
                for i in resp.result.bids {
                    client
                        .execute(
                            "INSERT INTO ftx_orderbook_15s_snapshots VALUES ($1, $2, $3, $4, $5)",
                            &[&time, &pair, &i.price, &i.size, &true],
                        )
                        .await?;
                }
                // Ingest asks.
                for i in resp.result.asks {
                    client
                        .execute(
                            "INSERT INTO ftx_orderbook_15s_snapshots VALUES ($1, $2, $3, $4, $5)",
                            &[&time, &pair, &i.price, &i.size, &false],
                        )
                        .await?;
                }
                println!("Ingested {} at {}", pair, time);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
