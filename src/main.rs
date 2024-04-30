mod binance_helper;
mod config_api;
mod excel_helper;
mod position;
mod trade;
mod utils;

use crate::binance_helper::create_binance_client;
use crate::config_api::Config;
use crate::excel_helper::write_to_excel;
use crate::position::Position;
use crate::trade::Trade;
use crate::utils::*;

use std::collections::{BTreeSet, HashSet};

use std::io;

fn main() {
    let config = match Config::new() {
        Ok(value) => {
            println!("Keys successfully read");
            value
        }
        Err(err) => {
            panic!("The error occured while reading keys: {err}");
        }
    };
    println!("Enter time period (today | yesterday | this week | YYYY-MM--DD): ");
    let mut date_str = String::new();
    io::stdin()
        .read_line(&mut date_str)
        .expect("failed to read from stdin");

    let timestamp_pairs = get_timestamp_mil2(&date_str.trim());

    // let config = Config::new().expect("Failed to read configuration file");

    let fapi_client = create_binance_client(&config);

    let mut all_trades: Vec<Trade> = Vec::<Trade>::new();
    for (start_time, end_time) in timestamp_pairs {
        let unique_symbols: BTreeSet<String> =
            get_unique_symbols(&fapi_client, start_time, end_time);
        for symbol in &unique_symbols {
            print!("{}\t\t", symbol);
        }
        println!();

        for ticker in &unique_symbols {
            let client_trade: Vec<Trade> =
                get_symbol_trades(&fapi_client, ticker.clone(), start_time, end_time, 1000);
            all_trades.extend(client_trade);
        }
    }
    let mut positions = Position::make_positions(&mut all_trades);
    for pos in &positions {
        println!("{:?}", pos);
    }
    // Remove duplicates
    let mut unique_positions = HashSet::new();
    let mut dedup_positions = Vec::new();

    for pos in positions {
        let key = (pos.symbol.clone(), pos.side.clone(), pos.time_start);
        if !unique_positions.contains(&key) {
            unique_positions.insert(key);
            dedup_positions.push(pos);
        }
    }
    write_to_excel(&mut dedup_positions);
}
