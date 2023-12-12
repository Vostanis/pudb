# Required Dependencies
```
docker-compose
```

# Pop-up Database (P-U DB)
Ready-made PostgreSQL config for Finance-related data;

## 1. SEC Company Facts
    _ large file, ~20GB, downloaded, unzipped, then formatted & placed in local pgsql db

## 2. Finnhub
    _ candlesticks (OHLCV)
    _ IPO calendar
    _ earnings calendar
    _ earnings surprises
    _ crypto (symbols, candles)
    _ patents
    _ senate lobbying
    _ VISA applications

# To do
- [ ] finnhub api
    - [ ] schema
    - [ ] url & json navigation

- [ ] develop webscraper.rs
    - [ ] proc macros for get_vec() (threads & headers)
    - [ ] status enum
    - [ ] continue to add to struct (plan for reusability)

- [ ] implement Webscraper on PU-DB[^1]: 
    domains:
        - [ ] google
        - [ ] bloomberg
        - [ ] financial times
        - [ ] reuters
        - [ ] the times
        - [ ] business insider
    
[^1]: aim for:
    _ references to name/ticker symbols
    _ statistics
    _ date
