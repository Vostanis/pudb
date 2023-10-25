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
