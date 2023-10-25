# Pop-up Database (P-U DB)
Ready-made PostgreSQL config for Finance-related data;
	1. SEC Company Facts
        - large file, ~20GB, downloaded, unzipped, then formatted & placed in local pgsql db
    2. Finnhub
        - candlesticks (OHLCV)
        - IPO calendar
        - earnings calendar
        - earnings surprises
        - crypto (symbols, candles)
        - patents
        - senate lobbying
        - VISA applications

# Further Updates
    a. updated src/ directory format;
        src/
            engine.rs   -- generalised functions, used across each api
            config.rs   -- personal details; user-agent email etc.
            sec.rs
            finnhub.rs

    b. news webscraper
        domains:
            - google
            - bloomberg
            - financial times
            - reuters
            - the times
            - business insider
        aim for:
            - references to name/ticker symbols
            - statistics
            - date
        directory:
        src/webscraper/
            engine.rs
            schema.rs
