use serde::{Serialize, Deserialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub auth: Auth,
    pub keys: Keys,
    pub server: Server,
    pub database: Database,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    pub user_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Keys {
    pub finnhub: String,
    pub mexc: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub name: String,
    pub username: String,
    pub password: String,
}

pub fn load_toml(file: &str) -> Config {
    let mut file = File::open("config.toml")
        .expect("ERROR! Failed to open file");
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("ERROR! Failed to read file");
    
    let toml: Config = toml::from_str(&contents)
        .expect("ERROR! Failed to read .toml from String");

    toml
}
