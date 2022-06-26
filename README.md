# Data Scraper

This is a (personal) small binary to scrape k-line data from Binance (1m) and FTX (15s) and insert it into a PostgreSQL database (more specifically, a TimeScaleDB database). Support for other exchanges is planned, and will most likely be implemented around the end of June 2022.

**Note:** There seems to be an issue with DNS servers such that due to the number of requests being sent to the DNS server, some datapoints may get blocked. Still looking into how to fix that issue.

## FTX Orderbook Snapshots.

To download FTX orderbooks (depth=20, can be changed in the code) every 15s (can be changed in the code), use the following command:

```shell
cargo run -r --bin ftx-orderbook {list of tickers}
```

The list of tickers should be in the form: "BASE_CURR QUOTE_CURR". Here is a following example:

```shell
cargo run -r --bin ftx-orderbook "BTC USD" "BTC USDT" "USDT USD"
```

### Schema

The SQL command to create the TimeScaleDB table compatible with the script is:

```sql
CREATE TABLE ftx_orderbook_15s_snapshots (
	time TIMESTAMPTZ NOT NULL,
	pair TEXT NOT NULL,
	price DOUBLE PRECISION NOT NULL,
	size DOUBLE PRECISION NOT NULL,
	is_bid BOOL NOT NULL
);

CREATE INDEX idx_pair_time ON ftx_orderbook_15s_snapshots (pair ASC, time DESC);
```
