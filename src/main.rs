#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused)]

pub mod config;
pub mod engine;
pub mod schema;

// use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

// enter docker psql; terminal command:
// sudo docker exec -it sec_api psql -U postgres -d postgres -p 5432 -W

#[tokio::main]
async fn main() {
    // Load config variables
    let config = config::load_toml("config.toml");

    // Bulk GET requests
    let urls = vec![
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
        // "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip",
        "https://www.sec.gov/files/company_tickers.json",
    ];
    engine::bulk_url_download(urls, &config.auth.user_agent, 3).await;
    engine::unzip("./data/companyfacts.zip", "./data/facts").await;

    // [.zip -> pgsql] migration
    println!("Initialising PostgreSQL tables ...");
    engine::pg_init(
        &config.server.host, 
        &config.server.port, 
        &config.database.username, 
        &config.database.name, 
        &config.database.password
    ).await;

    // Load the list of all Company Names/Tickers
    let companies = engine::read_json_file::<HashMap<String, schema::SecCompanies>>(
        "./data/company_tickers.json",
    ).await.expect("ERROR! Failed to read SEC company list");

    // Copy .json files to PostgreSQL equivalents. ~35 concurrent threads seems best fit
    let semaphore = Arc::new(Semaphore::new(35));
    let mut handles = vec![];
    for (_key, company) in companies {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let config_copy = config.clone();
        handles.push(tokio::spawn(async move {
            let facts_paths = format!("./data/facts/CIK{:010}.json", &company.cik_str);
            println!("Inserting values for: {} - {}", &company.ticker, &company.title);
            engine::pg_dump(
                &facts_paths, 
                &config_copy.server.host, 
                &config_copy.server.port, 
                &config_copy.database.username, 
                &config_copy.database.name, 
                &config_copy.database.password
            ).await;
            drop(permit);
        }));
        for handle in &handles {
            handle;
        }
    }
}
