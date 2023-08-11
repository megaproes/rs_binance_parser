use binance::errors::Error;
use binance::futures::*;
use binance::{api::*, market::Market};
use chrono::{NaiveDate, TimeZone, Utc};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Default)]
struct Trade<'a> {
    symbol: &'a str,
    side: &'a str,
    price: f64,
    qty: f64,
    realized_pnl: f64,
    quote_qty: f64,
    commission: f64,
    time: u64,
}
#[derive(Default)]
struct Position<'a> {
    symbol: &'a str,
    side: &'a str,
    average_entry_price: f64,
    average_exit_price: f64,
    realized_pnl_net: f64,
    realized_pnl_gross: f64,
    commission: f64,
    volume_dollar: f64,
    volume_quantity: f64,
    exit_volume_quantity: f64,
    time_start: u64,
    time_finished: u64,
}

fn main() {
    let date_str = "2023-07-25";
    let api_key = String::from("");
    let secret_key =
        String::from("");
    // let market: Market = Binance::new(None, None);

    let fapi_client: account::FuturesAccount =
        binance::futures::account::FuturesAccount::new(Some(api_key), Some(secret_key));

    let unique_symbols: BTreeSet<String> = get_unique_symbols(&fapi_client, &date_str);
    for symbol in unique_symbols.clone() {
        print!("{}\t\t", symbol);
    }
    
    
    let all_trades: Vec<Trade> = Vec::<Trade>::new();
    for ticker in unique_symbols  {
        let client_trade = get_symbol_trades(&fapi_client, ticker, get_timestamp_mil(date_str).unwrap(), 0, 1000);
        // let trades = Trade::make_trades();
        // let json_string = serde_json::to_string(&trades);
    }
}

fn get_timestamp_mil(date_str: &str) -> Option<u64> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Failed to parse data . . .");
    let datetime = Utc
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let timestamp_millis = datetime.timestamp_millis();
    match timestamp_millis {
        0 => None,
        _ => Some(timestamp_millis as u64),
    }
}
fn get_unique_symbols(fapi_client: &account::FuturesAccount, date_str: &str) -> BTreeSet<String> {
    let income_req: account::IncomeRequest = account::IncomeRequest {
        symbol: None,
        income_type: None,
        start_time: get_timestamp_mil(date_str),
        end_time: None,
        limit: Some(1000),
    };

    let income_history_json: Result<Vec<model::Income>, Error> = fapi_client.get_income(income_req);

    let json_string = serde_json::to_string(&income_history_json.unwrap()); // println!("{:?}", income_history);
    let parsed_income: Value = serde_json::from_str(&json_string.unwrap()).unwrap();

    let mut symbols = BTreeSet::new();
    for symbol in parsed_income.as_array().unwrap() {
        symbols.insert(
            symbol
                .as_object()
                .unwrap()
                .get("symbol")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        );
    }
    return symbols;
}

fn get_symbol_trades(fapi_client: &account::FuturesAccount, symbol: String, start_time: u64, end_time: u64, limit: u16) -> Vec<Trade> {
    let json_trades: Result<Vec<model::TradeHistory>, Error> = 
    fapi_client.get_user_trades( symbol, None, Some(start_time), None, Some(limit));
    println!("{:?}\n\n", &json_trades);
   
    let json_string = serde_json::to_string(&json_trades.unwrap());
    let parsed_income: Value = serde_json::from_str(&json_string.unwrap()).unwrap();
    
    let trades = Vec::<Trade>::new();
    for trade in parsed_income.as_array().unwrap() {
        let temp = trade.as_object().unwrap();
        let _symbol = temp.get("symbol").unwrap().as_str().unwrap();
        let _side = temp.get("side").unwrap().as_str().unwrap();
        let _price = temp.get("price").unwrap().as_str().unwrap();
        let _qty = temp.get("qty").unwrap().as_str().unwrap();
        let _realized_pnl = temp.get("realized_pnl").unwrap().as_str().unwrap();
        
        let _side = temp.get("side").unwrap().as_str().unwrap();
        
        
        trades.push(Trade { symbol: _symbol, side: side, price: (), qty: (), realized_pnl: (), quote_qty: (), commission: (), time: () })
    }
   
    trades
}

impl Trade<'static> {
    // fn make_trades(unique_symbols: BTreeSet<String>, fapi_client: &account::FuturesAccount) -> Vec<Trade<'static>> {
       
    // }
}

impl Position<'static> {
    fn make_positions(trades: Vec<Trade>) -> Vec<Position> {
        let mut positions = Vec::<Position>::new();
        let mut i = 0;
        while i < trades.len() {
            let mut pos = Position::default();
            pos.symbol = trades[i].symbol;
            pos.side = trades[i].symbol;

            pos.average_entry_price += trades[i].price * trades[i].qty;
            pos.volume_quantity += trades[i].qty;
            pos.volume_dollar += trades[i].quote_qty;

            pos.time_start = trades[i].time;
            pos.commission += trades[i].commission;

            let mut j = i + 1;
            while j < trades.len() {
                if pos.side == trades[j].side {
                    pos.average_entry_price += trades[j].price * trades[j].qty;
                    pos.volume_quantity += trades[j].qty;
                    pos.volume_dollar += trades[j].quote_qty;
                    pos.commission += trades[j].commission;
                } else if pos.side != trades[j].side {
                    pos.average_exit_price += trades[j].price * trades[j].qty;
                    pos.exit_volume_quantity += trades[j].qty;
                    pos.realized_pnl_net += trades[j].realized_pnl;
                    pos.commission += trades[j].commission;
                    if (pos.volume_quantity - pos.exit_volume_quantity).abs() < 0.000001 {
                        pos.average_entry_price /= pos.volume_quantity;
                        pos.average_exit_price /= pos.exit_volume_quantity;
                        pos.realized_pnl_gross = pos.realized_pnl_net - pos.commission;

                        i = j;
                        break;
                    }
                }
            }
            positions.push(pos);
        }

        positions
    }
}
