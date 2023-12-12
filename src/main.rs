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

    // Create a data directory
    if !std::path::Path::new("./data").exists() {
        println!("Creating data directory ...");
        std::fs::create_dir("./data").unwrap();
    }

    // run docker-compose.yaml
    let docker_output = std::process::Command::new("docker-compose")
        .arg("-f")
        .arg("docker-compose.yaml")
        .arg("up")
        .arg("-d")
        .output()
        .expect("EXPECTED psql Docker container to run");
    println!("STATUS: {}", docker_output.status);

    // fetch (& prepare) data sources
    // println!("Fetching data ...");
    // sec::fetch_data(&config.auth.user_agent).await;

    // create (or reset) psql tables
    println!("Initialising PostgreSQL tables ...");
    engine::psql_init(&config).await;

    // move data from files to psql
    println!("Migrating data to PostgreSQL ...");
    sec::psql_en_masse(&config).await;
}
