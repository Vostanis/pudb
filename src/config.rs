use serde::{Serialize, Deserialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    auth: Auth,
    keys: Keys,
    server: Server,
    database: Database,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    user_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Keys {
    finnhub: String,
    mexc: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    host: String,
    port: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    name: String,
    username: String,
    password: String,
}

pub fn load_toml(file: &str) {
    let mut file = File::open("config.toml")
        .expect("ERROR! Failed to open file");
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("ERROR! Failed to read file");
    
    let toml: Config = toml::from_str(&contents)
        .expect("ERROR! Failed to read .toml from String");
}
