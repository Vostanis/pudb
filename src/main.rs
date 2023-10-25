#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused)]

pub mod config;
pub mod engine;
pub mod sec;

// enter docker psql; terminal command:
// sudo docker exec -it sec_api psql -U postgres -d postgres -p 5432 -W

#[tokio::main]
async fn main() {
    // Load config variables
    let config = config::load_toml("config.toml");

    // fetch (& prepare) data sources
    println!("Fetching data ...");
    sec::fetch_data(&config.auth.user_agent).await;

    // create (or reset) psql tables
    println!("Initialising PostgreSQL tables ...");
    engine::psql_init(&config).await;

    // move data from files to psql
    println!("Migrating data to PostgreSQL ...");
    sec::psql_en_masse(&config).await;
}
