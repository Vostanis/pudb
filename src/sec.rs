use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::{engine, config::Config};


/*  KNOWN ERRORS
    "No such file or directory exists (os error 2)"
    i.e. company exists in company_tickers.json, but not within .zip
        NOT SURE WHY
            Maybe .zip is out of date?
            Found code MXRX was a file that doesn't exist - looks to be pre-IPO
*/

// download datasets
pub async fn fetch_data(user_agent: &str) {
    let urls = vec![
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
        // "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip",   // needs work
        "https://www.sec.gov/files/company_tickers.json",
    ];
    engine::bulk_url_download(urls, user_agent, 3).await;
    engine::unzip("./data/companyfacts.zip", "./data/facts").await;
}

// single-thread version of psql insertion
pub async fn psql(file_path: &str, config: &Config) { // NOT A FAN OF THIS: TOO LONG - generalise

    // set config "let"s so format! is usable
    let host = &config.server.host;
    let port = &config.server.port;
    let user = &config.database.username;
    let dbname = &config.database.name;
    let password = &config.database.password;

    let config_str = format!("host={host} port={port} user={user} dbname={dbname} password={password}"); 
    let (client, connection) = tokio_postgres::connect(
        &config_str,
        tokio_postgres::NoTls,
    )
    .await
    .expect("ERROR! Could not connect to database");

    // single task to handle connection (error)
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("ERROR! Connection failed: {}", e);
        }
    });

    match engine::read_json_file::<SecCompany>(file_path).await {
        Ok(data) => {
            if let Some(us_gaap_map) = &data.facts.us_gaap {
                // handle us_gaap: Option<BTreeMap>
                for metric in us_gaap_map.keys() {
                    // since "unit of measurement" is stored as a key (in the struct architecture),
                    // we do the following to obtain it. this is then to be placed into our pgsql table
                    let unit: String = {
                        let units_vec: Vec<_> = us_gaap_map[metric].units.keys().cloned().collect();
                        units_vec[0].clone()
                    };

                    // let sql =
                    let _sql = client
                        .query(
                            "
                            INSERT INTO raw.sec_data (
                                cik, entity_name, label, description, unit, 
                                start_date, end_date, val, accn, fy, 
                                fp, form, filed, frame
                            )
                            VALUES (
                                $1, $2, $3, $4, $5, 
                                $6, $7, $8, $9, $10,
                                $11, $12, $13, $14
                            )   
                        ",
                            &[
                                &data.cik,
                                &data.entity_name,
                                &us_gaap_map[metric].label,
                                &us_gaap_map[metric].description,
                                &unit,
                                &us_gaap_map[metric].units[&unit][0].start_date,
                                &us_gaap_map[metric].units[&unit][0].end_date,
                                &us_gaap_map[metric].units[&unit][0].val,
                                &us_gaap_map[metric].units[&unit][0].accn,
                                &us_gaap_map[metric].units[&unit][0].fy,
                                &us_gaap_map[metric].units[&unit][0].fp,
                                &us_gaap_map[metric].units[&unit][0].form,
                                &us_gaap_map[metric].units[&unit][0].filed,
                                &us_gaap_map[metric].units[&unit][0].frame,
                            ],
                        )
                        .await
                        .expect("ERROR! Failed to complete INSERT query for SEC company fact data");
                }
            }
        }
        Err(e) => eprintln!("ERROR! Failed to read {}; {}", file_path, e),
    }
}

// multithreaded version of psql insertion
pub async fn psql_en_masse(config: &Config) {

    let companies = engine::read_json_file::<HashMap<String, SecCompanies>>(
        "./data/company_tickers.json",
    ).await.expect("ERROR! Failed to read SEC company list");

    // Copy .json files to PostgreSQL equivalents. ~35 concurrent threads seems best fit
    // Should play around with dynamism in threading
    let semaphore = Arc::new(Semaphore::new(35));
    let mut handles = vec![];
    for (_key, company) in companies {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let config_copy = config.clone();
        handles.push(tokio::spawn(async move {
            let facts_paths = format!("./data/facts/CIK{:010}.json", &company.cik_str);
            println!("Inserting values for: {} - {}", &company.ticker, &company.title);
            psql(
                &facts_paths, 
                &config_copy
            ).await;
            drop(permit);
        }));
        for handle in &handles {
            handle;
        }
    }
}

// CIK code can either be a 10-digit string, or shortened number; de_cik handles both
fn de_cik<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // general deserialisation, followed by match statement (depending on type found)
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        // if it's a num, pad it as a 10-char string
        serde_json::Value::Number(num) => {
            if let Some(i32_value) = num.as_i64() {
                // as_i64() does the same job for i32
                Ok(format!("{:010}", i32_value))
            } else {
                Err(serde::de::Error::custom(
                    "ERROR! Unable to parse i32 from JSON",
                ))
            }
        }

        // if it's a string, then Ok()
        serde_json::Value::String(s) => Ok(s),

        // else return an error (it can't be correct type)
        _ => Err(serde::de::Error::custom("ERROR! Invalid type for CIK")),
    }
}

// company ticker overview
#[derive(Debug, Deserialize)]
pub struct SecCompanies {
    pub cik_str: i32,
    pub ticker: String,
    pub title: String,
}

// core data
#[derive(Debug, Deserialize)]
pub struct SecCompany {
    #[serde(deserialize_with = "de_cik")]
    pub cik: String,
    #[serde(rename = "entityName")]
    pub entity_name: String,
    pub facts: Facts,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")] // handles "us-gaap" column name
pub struct Facts {
    pub us_gaap: Option<BTreeMap<String, Metric>>,
    pub dei: Option<BTreeMap<String, Metric>>,
}

#[derive(Debug, Deserialize)]
pub struct Metric {
    pub label: Option<String>,
    pub description: Option<String>,
    pub units: BTreeMap<String, Vec<Values>>,
}

#[derive(Debug, Deserialize)]
pub struct Values {
    #[serde(rename = "start")] // renamed to match "end" keyword issue below
    pub start_date: Option<String>,
    #[serde(rename = "end")] // keyword in pgsql; best to rename
    pub end_date: Option<String>,
    pub val: Option<f64>,
    pub accn: Option<String>,
    pub fy: Option<i32>,
    pub fp: Option<String>,
    pub form: Option<String>,
    pub filed: Option<String>,
    pub frame: Option<String>,
}
