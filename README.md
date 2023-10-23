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
    a. potential "src" directory format:
        src/
            engine.rs
            config.rs
            sec/
                engine.rs
                schema.rs
            finnhub/
                engine.rs
                schema.rs
            mexc/
                engine.rs
                schema.rs

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
