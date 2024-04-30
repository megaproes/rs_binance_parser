use serde::Deserialize;
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
        let config: Config = serde_json::from_str(&contents)?;

        let api_key_len = config.api_key.chars().count();
        let secret_key_len = config.secret_key.chars().count();
        
        let mut errors = Vec::new();
        
        if api_key_len < 64 {
            errors.push(format!("API key should be 64 characters, but it has {} characters", api_key_len));
        }
        if secret_key_len < 64 {
            errors.push(format!("Secret key should be 64 characters, but it has {} characters", secret_key_len));
        }
        
        if !errors.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                errors.join(", and "),
            ));
        }

        Ok(config)
    }
}
