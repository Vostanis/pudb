// 30 calls per sec: code 429 if exceeded
// https://finnhub.io/api/v1/

let base_url = "https://finnhub.io/api/v1/";

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockSymbol {
    currency: String,
    description: String,
    display_symbol: String,
    figi: String,
    mic: String,
    symbol: String,
    #[serde(rename = "type")] // type is a Rust keyword
    stock_type: String,
}
