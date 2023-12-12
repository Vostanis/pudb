# Required Dependencies
docker-compose

# Pop-up Database (P-U DB)
Ready-made PostgreSQL config for Finance-related data;

## 1. SEC Company Facts
+ large file, ~20GB, downloaded, unzipped, then formatted & placed in local pgsql db

## 2. Finnhub
+ candlesticks (OHLCV)
+ IPO calendar
+ earnings calendar
+ earnings surprises
+ crypto (symbols, candles)
+ patents
+ senate lobbying
+ VISA applications

# To do
- [ ] finnhub api
    - [ ] schema
    - [ ] url & json navigation

- [ ] develop webscraper.rs
    - [ ] proc macros for get_vec() (threads & headers)
    - [ ] status enum
    - [ ] continue to add to struct (plan for reusability)

- [ ] implement Webscraper on PU-DB: 
    - [ ] domains:
        - [ ] google
        - [ ] bloomberg
        - [ ] financial times
        - [ ] reuters
        - [ ] the times
        - [ ] business insider
 
aim for:
+ references to name/ticker symbols
+ statistics
+ date
