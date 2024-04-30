use binance::{api::Binance, futures::account::FuturesAccount};
use crate::config_api::Config;
pub fn create_binance_client(config: &Config) -> FuturesAccount {
    FuturesAccount::new(Some(config.api_key.clone()), Some(config.secret_key.clone()))
}