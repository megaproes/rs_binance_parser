use serde::{Deserialize};

use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct Config {
    pub api_key: String,
    pub secret_key: String,
}

impl Config {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut file = File::open("src/config.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}