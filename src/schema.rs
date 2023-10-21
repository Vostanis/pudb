use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashMap};

/*  ERRORS HANDLED
    ==============

    1) CIK code can either be 10-char string or integer
        => de_cik() function handles both

    2) Datasets can be missing US-GAAP or DEI (or both) data
        => us_gaap/dei: Option<BTreeMap> (makes data handling more annoying but worth it)

    3) "No such file or directory exists (os error 2)"
        NOT SURE WHY
            Maybe .zip is out of date?
            Found code MXRX was a file that doesn't exist - looks to be pre-IPO
*/

// SEC
// ============================================

// menu data
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
    #[serde(rename = "start")] // renamed to suit "end" keyword issue below
    pub start_date: Option<String>,
    #[serde(rename = "end")] // keyword in pgsql - best to rename
    pub end_date: Option<String>,
    pub val: Option<f64>,
    pub accn: Option<String>,
    pub fy: Option<i32>,
    pub fp: Option<String>,
    pub form: Option<String>,
    pub filed: Option<String>,
    pub frame: Option<String>,
}

// Functions for error handling
// ============================

fn de_cik<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // if confused: refer to serde_json::Value documentation online (it's en enum of all the types!)
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

        // if it's a string, then we good homie
        serde_json::Value::String(s) => Ok(s),

        // else return an error (it can't be correct type)
        _ => Err(serde::de::Error::custom("ERROR! Invalid type for CIK")),
    }
}
