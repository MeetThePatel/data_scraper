* Data Scraper

This is a (personal) small binary to scrape k-line data from Binance and insert it into a PostgreSQL database (more specifically, a TimeScaleDB database). Support for other exchanges is planned, and will most likely be implemented around the end of June 2022.

**Note:** There seems to be an issue with DNS servers such that due to the number of requests being sent to the DNS server, some datapoints may get blocked. Still looking into how to fix that issue.