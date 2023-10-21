#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused)]

pub mod engine;
pub mod schema;

use anyhow::Result;
use dotenv::dotenv;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

// enter docker psql; terminal command:
// sudo docker exec -it sec_api psql -U postgres -d postgres -p 5432 -W

#[tokio::main]
async fn main() {
    // load env vars
    dotenv().ok();
    let user_agent = std::env::var("EMAIL").unwrap();
    let finnhub_api_key = std::env::var("FINNHUB_API_KEY").unwrap();
    let finnhub_auth = std::env::var("FINNHUB_AUTH").unwrap();

    // Bulk GET requests
    // =================
    let urls = vec![
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
        // "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip",
        "https://www.sec.gov/files/company_tickers.json",
    ];
    engine::bulk_url_download(urls, , 3).await;
    engine::unzip("./data/companyfacts.zip", "./data/facts").await;

    // [.zip -> pgsql] migration
    // =========================
    println!("Initialising PostgreSQL tables ...");
    engine::pg_init().await;

    let companies = engine::read_json_file::<HashMap<String, schema::SecCompanies>>(
        "./data/company_tickers.json",
    )
    .await
    .expect("ERROR! Failed to read SEC company list");

    let semaphore = Arc::new(Semaphore::new(35));
    let mut handles = vec![];
    for (_key, company) in companies {
        // possible async/task_spawn upgrade here?
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            let facts_paths = format!("./data/facts/CIK{:010}.json", &company.cik_str);
            println!(
                "Inserting values for: {} - {}",
                &company.ticker, &company.title
            );
            engine::pg_dump(&facts_paths).await;
            drop(permit);
        }));

        for handle in &handles {
            handle;
        }
    }

    // FinnHub
    // =======
    // https://finnhub.io/api/v1/stock/candle?symbol={ticker}&resolution=1&from=1693493346&to=1693752546&token={api_key}
    // 30 calls per sec: code 429 if exceeded
}

// CIK0001908984 failed to read file

/* NOTES */
// let tickers = {
//     let companies = engine::read_json_file::<HashMap<String, schema::SecCompanies>>("./data/company_tickers.json")
//         .await
//         .unwrap();
//     let mut tickers: HashMap<String, String> = HashMap::new();
//     for (_key, value) in &companies {
//         tickers.insert(
//             value.ticker.clone(), format!("{:0>10}", value.cik_str.clone().to_string())
//         );
//     }
//     tickers
// };
//
// let cik_resp = {
//     loop {
//         let mut ticker_input = String::new();
//         println!("Ticker:");
//         std::io::stdin()
//             .read_line(&mut ticker_input)
//             .expect("Failed to read input line.");
//         let ticker_input = ticker_input.trim().to_uppercase();
//         match tickers.get(&ticker_input) {
//             Some(cik_str) => {
//                 break cik_str
//             },
//             None => println!("Ticker not found."),
//         }
//     }
// };
//
// let facts_paths = format!("./data/facts/CIK{cik_resp}.json");
// let company = engine::read_json_file::<schema::SecCompany>(&facts_paths).await.unwrap(); // ERROR HANDLING NEEDED!!!
// &company.facts.dei OR &company.facts.us_gaap are HashMaps
// i.e., accessing further granularity requires .get() or .keys();
// println!("{:#?}", &company.facts.dei.keys());
// println!("{:#?}", &company.facts.us_gaap.keys());
// println!("{:#?}", &company.facts.us_gaap["DebtCurrent"].units["USD"][0].start);
// println!("{:#?}", &company.cik);
// println!("{:#?}", &company.entity_name);
// println!("{:#?}", &company);
