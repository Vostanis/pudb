use futures::stream::StreamExt;
use serde::Deserialize;
use std::{
    error::Error,
    io,
    io::Read, // not sure why Rust is insisting on this separate import for futures?
    fs,
};

use crate::sec::{self, SecCompany};

// GENERALISED FUNCTIONS

// 1. bulk url download of a vector of String URLs, with n (multi)threads
pub async fn bulk_url_download(
    endpoints: Vec<&str>, 
    user_agent: &str, 
    n_threads: usize
) {
    let client = reqwest::Client::new();
    futures::stream::iter(endpoints.into_iter().map(|url| {
        // model the request as a Future
        let future = client
            .get(url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .send();

        async move {
            println!("Requesting {} ...", url);
            match future.await {
                Ok(response) => {
                    // Status Code has to be called upon here to be referenced later
                    let status = response.status();
                    match response.bytes().await {
                        Ok(bytes) => match std::path::Path::new(url).file_name() {
                            Some(file_name) => {
                                let file_path = std::path::Path::new("./data").join(file_name);
                                let _ = tokio::fs::write(file_path, bytes).await;
                                println!("File written: {:?}", file_name);
                            }
                            None => println!(
                                "ERROR! Unable to retrieve file 
                                                     from URL: {url}"
                            ),
                        },
                        Err(_) => println!(
                            "ERROR! Unable to retrieve bytes for 
                                                endpoint: {url}; STATUS CODE: {status}"
                        ),
                    }
                }
                Err(_) => println!(
                    "ERROR! Unable to retrieve HTTPS response 
                                       for URL: {url}"
                ),
            }
        }
    }))
    .buffer_unordered(n_threads)
    .collect::<Vec<()>>()
    .await;
}

// 2. unzip file exttracts to target directory
pub async fn unzip(zip_file_path: &str, target_dir: &str) {
    match std::fs::File::open(zip_file_path) {
        Ok(file) => {
            let buf_reader = std::io::BufReader::new(file);
            match zip_extract::extract(buf_reader, std::path::Path::new(target_dir), false) {
                Ok(_) => println!("Extracting file: {zip_file_path}"),
                Err(_) => println!(
                    "ERROR! Could not extract file {} to target directory {}",
                    zip_file_path, target_dir
                ),
            }
        }
        Err(_) => println!("ERROR! Unable to open file {zip_file_path}"),
    }
}

pub async fn read_json_file<T: serde::de::DeserializeOwned>(
    file_path: &str,
) -> Result<T, Box<dyn Error + Send + Sync>> {
    let mut file = std::fs::File::open(file_path)?;
    let mut file_str = String::new();
    file.read_to_string(&mut file_str)?;

    let json: T = serde_json::from_str(&file_str)?;
    Ok(json)
}

// 3. initialise pgsql tables
pub async fn psql_init(host: &str, port: &str, user: &str, dbname: &str, password: &str) {
    let config_str = format!("host={host} port={port} user={user} dbname={dbname} password={password}"); 
    let (client, connection) = tokio_postgres::connect(
        &config_str,
        tokio_postgres::NoTls,
    )
    .await
    .expect("ERROR! Could not connect");

    // single task to handle connection (error)
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("ERROR! Connection failed: {}", e);
        } else {
            println!("Initial connection succesful")
        }
    });

    // initiliase tbls for inserting into later
    let queries: Vec<&str> = vec![
        "CREATE SCHEMA IF NOT EXISTS raw",
        "DROP TABLE IF EXISTS raw.sec_data",
        "CREATE TABLE raw.sec_data (
            cik             VARCHAR,
            entity_name     VARCHAR,
            label           VARCHAR,
            description     VARCHAR,
            unit            VARCHAR,
            start_date      VARCHAR,
            end_date        VARCHAR,
            val             FLOAT,
            accn            VARCHAR,
            fy              INTEGER,
            fp              VARCHAR,
            form            VARCHAR,
            filed           VARCHAR,
            frame           VARCHAR
        )",
        // "SELECT * FROM raw.sec_data"
    ];
    for query in queries {
        let _init_tbl_sql = client
            .query(query, &[])
            .await
            .expect("ERROR! Client failed to query PostgreSQL database");
        println!("{:#?}", _init_tbl_sql);
    }
}

// 4. insert into all files into docker-compose: postgres
pub async fn pg_dump(file_path: &str, host: &str, port: &str, user: &str, dbname: &str, password: &str) {
    let config_str = format!("host={host} port={port} user={user} dbname={dbname} password={password}"); 
    let (client, connection) = tokio_postgres::connect(
        &config_str,
        tokio_postgres::NoTls,
    )
    .await
    .expect("ERROR! Could not connect");

    // single task to handle connection (error)
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("ERROR! Connection failed: {}", e);
        }
    });

    match read_json_file::<schema::SecCompany>(file_path).await {
        Ok(data) => {
            if let Some(us_gaap_map) = &data.facts.us_gaap {
                // handle us_gaap: Option<BTreeMap>
                for metric in us_gaap_map.keys() {
                    // since "unit of measurement" is stored as a key (in the struct architecture),
                    // we do the following to obtain it, to then be place it into our pgsql table
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
