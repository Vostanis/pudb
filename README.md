# Pop-up Database (P-U DB)
Ready-made PostgreSQL config for Finance-related data;
	1. SEC Company Facts
        - large file, ~20GB, downloaded, unzipped, then formatted & placed in local pgsql db
    2. Finnhub
    3. MEXC

# Further Updates
    a. potential "src" directory format:
        src/
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
        aim for:
            - references to name/ticker symbols
            - statistics
            - date
