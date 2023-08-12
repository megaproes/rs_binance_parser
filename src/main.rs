use binance::errors::Error;
use binance::futures::*;
use binance::{api::*};
use chrono::{NaiveDate, TimeZone, Utc};
use serde_json::Value;
use std::collections::BTreeSet;

extern crate simple_excel_writer;
use simple_excel_writer as excel;

use excel::*;

#[derive(Default)]
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    qty: f64,
    realized_pnl: f64,
    quote_qty: f64,
    commission: f64,
    time: u64,
}

#[derive(Default, Debug)]
struct Position {
    symbol: String,
    side: String,
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
    let date_str = "2023-08-12";
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
    println!();
    let mut all_trades: Vec<Trade> = Vec::<Trade>::new();
    for ticker in unique_symbols {
        let client_trade: Vec<Trade> = get_symbol_trades(
            &fapi_client,
            ticker,
            get_timestamp_mil(date_str.clone()).unwrap(),
            get_timestamp_mil("2023-08-13").unwrap(),
            1000,
        );
        all_trades.extend(client_trade);
    }

    let positions = Position::make_positions(&mut all_trades);
    for pos in &positions {
        println!("{:?}", pos);
    }
    write_to_excel(positions)

}
fn write_to_excel(positions: Vec<Position>) {
     let mut wb = Workbook::create("output_positions.xlsx");
    let mut sheet = wb.create_sheet("Positions");

    wb.write_sheet(&mut sheet,|sheet_writer| 
        {
        let sw = sheet_writer;
        sw.append_row(row!["Date", "Time entry", "Time exit", "Ticker", "L / S", "Average Entry", "Average Exit", "Volume", "$Volume",
        "Commision", "P / L Gross", "P / L NET"])?;

        for pos in positions 
        {
            sw.append_row(row![pos.time_start as f64, pos.time_start as f64, pos.time_finished as f64, 
            pos.symbol.to_string(), pos.side.to_string(), pos.average_entry_price as f64, pos.average_exit_price as f64, 
            pos.volume_quantity as f64, pos.volume_dollar as f64,
            pos.commission as f64, pos.realized_pnl_gross as f64, pos.realized_pnl_net as f64])?;
        }
        Ok(())  
    }).unwrap();

    wb.close().expect("close excel error!");
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
        end_time: get_timestamp_mil("2023-08-13"),
        limit: Some(1000),
    };

    let income_history_json: Result<Vec<model::Income>, Error> = fapi_client.get_income(income_req);
    //println!("{:?}", income_history_json);

    let json_string = serde_json::to_string(&income_history_json.unwrap()); 
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

fn get_symbol_trades(
    fapi_client: &account::FuturesAccount,
    symbol: String,
    start_time: u64,
    end_time: u64,
    limit: u16,
) -> Vec<Trade> {
    let json_trades: Result<Vec<model::TradeHistory>, Error> =
        fapi_client.get_user_trades(&symbol, None, Some(start_time), Some(end_time), Some(limit));
   // println!("{:?}\n\n", &json_trades);

    match json_trades {
        Ok(trade_histories) => {
            let mut trades = Vec::new();

            for trade_history in trade_histories {
                trades.push(Trade {
                    symbol: trade_history.symbol, 
                    side: trade_history.side,
                    price: trade_history.price,
                    qty: trade_history.qty,
                    realized_pnl: trade_history.realized_pnl,
                    quote_qty: trade_history.quote_qty,
                    commission: trade_history.commission,
                    time: trade_history.time,
                });
            }
            trades
        }
        Err(e) => {
            eprintln!("Error fetching trades: {:?}", e);
            Vec::new()
        }
    }
}

impl Position {
    fn make_positions(trades: &mut Vec<Trade>) -> Vec<Position> {
        let mut positions = Vec::new();

        let mut i = 0;
        while i < trades.len() {
            let mut pos = Position::default();

            pos.symbol = trades[i].symbol.clone();
            pos.side = trades[i].side.clone();

            pos.time_start = trades[i].time;
            pos.commission += trades[i].commission;

            pos.average_entry_price += trades[i].price * trades[i].qty;
            pos.volume_quantity += trades[i].qty;
            pos.volume_dollar += trades[i].quote_qty;

            let mut j = i + 1;
            while j < trades.len() {
                if pos.side == trades[j].side 
                {
                    pos.average_entry_price += trades[j].price * trades[j].qty;
                    pos.volume_quantity += trades[j].qty;
                    pos.volume_dollar += trades[j].quote_qty;
                    pos.commission += trades[j].commission;
                } 
                else 
                {
                    pos.average_exit_price += trades[j].price * trades[j].qty;
                    pos.exit_volume_quantity += trades[j].qty;

                    pos.realized_pnl_net += trades[j].realized_pnl;
                    pos.commission += trades[j].commission;

                    if (pos.volume_quantity - pos.exit_volume_quantity).abs() < 0.000001 {
                        pos.average_entry_price /= pos.volume_quantity;
                        pos.average_exit_price /= pos.exit_volume_quantity;
                        pos.realized_pnl_gross = pos.realized_pnl_net - pos.commission;
                        pos.time_finished = trades[j].time;
                        j += 1;
                        i = j;
                       
                        break;
                    }
                }
                j += 1;
            }

            positions.push(pos);
            i += 1;
        }

        positions
    }
}
