use binance::api::*;
use binance::errors::Error;
use binance::futures::*;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use std::collections::BTreeSet;

extern crate simple_excel_writer;
use simple_excel_writer as excel;

use excel::*;
use std::io;

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
    println!("Enter time period (today | yesterday | this week | YYYY-MM--DD): ");
    let mut date_str = String::new();
    io::stdin()
        .read_line(&mut date_str)
        .expect("failed to read from stdin");

    let timestamp_pairs = get_timestamp_mil2(&date_str.trim());

    let api_key = String::from("1Su30ANOkWEBHgJbJ4SYWOFMg2IFBXVbbyA0mojqPfHGoL65egDc64Fg9oIQU73j");
    let secret_key =
        String::from("ulMm5wpCmgvBEAlVRNvQbbMvIbcOIv4MYwag8Ba0ZhTSJJpFxczrfhYqLlDn04Md");

    let fapi_client: account::FuturesAccount =
        binance::futures::account::FuturesAccount::new(Some(api_key), Some(secret_key));

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
    write_to_excel(&mut positions);
}
fn get_timestamp_mil2(period: &str) -> Vec<(u64, u64)> {
    let ukraine_timezone = FixedOffset::east_opt(3 * 3600).unwrap();
    let current_time = Utc::now().with_timezone(&ukraine_timezone);
    let mut periods: Vec<(u64, u64)> = Vec::new();

    match period {
        "today" => {
            let start_of_day = current_time.date_naive().and_hms_opt(0, 0, 0);
            let end_time = current_time;
            periods.push((
                start_of_day.unwrap().timestamp_millis() as u64,
                current_time.timestamp_millis() as u64,
            ));
        }
        "yesterday" => {
            let start_of_yesterday =
                (current_time.date_naive() - Duration::days(1)).and_hms_opt(0, 0, 0);
            let end_of_yesterday =
                current_time.date_naive().and_hms_opt(0, 0, 0).unwrap() - Duration::seconds(1);

            periods.push((
                start_of_yesterday.unwrap().timestamp_millis() as u64,
                end_of_yesterday.timestamp_millis() as u64,
            ));
        }
        "this week" => {
            let days_since_monday = current_time.date_naive().weekday().num_days_from_monday();
            for i in 0..=days_since_monday {
                let date_minus_i_days = current_time.date_naive() - Duration::days(i.into());
                let start_of_day = date_minus_i_days.and_hms_opt(0, 0, 0);

                let end_of_day = if i == 0 {
                    current_time.naive_local()
                } else {
                    let date_minus_i_minus_one_days =
                        current_time.date_naive() - Duration::days((i - 1).into());
                    date_minus_i_minus_one_days.and_hms_opt(23, 59, 59).unwrap()
                };
                println!("start_of_day: {:?}  end_of_day: {:?}", start_of_day.clone(), end_of_day.clone());
                periods.push((
                    start_of_day.unwrap().timestamp_millis() as u64,
                    end_of_day.timestamp_millis() as u64,
                ));
            }
        }
        _ => {
            // If in format YYYY-MM--DD . . .
            if let Ok(date) = NaiveDate::parse_from_str(period, "%Y-%m-%d") {
                let start_of_day = ukraine_timezone
                    .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                    .unwrap();
                let end_of_day = ukraine_timezone
                    .from_local_datetime(&date.and_hms_opt(23, 59, 59).unwrap())
                    .unwrap();
                periods.push((
                    start_of_day.timestamp_millis() as u64,
                    end_of_day.timestamp_millis() as u64,
                ));
            }
        }
    }

    periods
}

fn write_to_excel(positions: &mut  Vec<Position>) {
        let ukraine_timezone = FixedOffset::east_opt(3 * 3600).unwrap();

        let current_time = Utc::now().with_timezone(&ukraine_timezone);
        
    positions.sort_by(|a, b| b.time_start.cmp(&a.time_start));

let mut wb = Workbook::create(format!("output_positions_{}.xlsx", current_time.format("%Y-%m-%d_%H-%M-%S")).as_str());
    let mut sheet = wb.create_sheet("Positions");

    wb.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row![
            "Date",
            "Time entry",
            "Time exit",
            "Ticker",
            "L / S",
            "Average Entry",
            "Average Exit",
            "Volume",
            "$Volume",
            "Commision",
            "P / L Gross",
            "P / L NET"
        ])?;

        for pos in positions {
            let time_start_dt =
                NaiveDateTime::from_timestamp_millis(pos.time_start as i64).unwrap();
            let time_finished_dt =
                NaiveDateTime::from_timestamp_millis(pos.time_finished as i64).unwrap();

            // Extract the date and time in the desired format
            let date = time_start_dt.date().format("%Y-%m-%d").to_string();
            let time_entry = time_start_dt.time().format("%H:%M:%S").to_string();
            let time_exit = time_finished_dt.time().format("%H:%M:%S").to_string();
            sw.append_row(row![
                date,
                time_entry,
                time_exit,
                pos.symbol.to_string(),
                pos.side.to_string(),
                pos.average_entry_price as f64,
                pos.average_exit_price as f64,
                pos.volume_quantity as f64,
                pos.volume_dollar as f64,
                pos.commission as f64,
                pos.realized_pnl_gross as f64,
                pos.realized_pnl_net as f64
            ])?;
        }
        Ok(())
    })
    .unwrap();

    wb.close().expect("close excel error!");
}

fn get_timestamp_mil(date_str: &str) -> Option<u64> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Failed to parse data . . .");
    let datetime = Utc
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    println!("{:?}", datetime);
    let timestamp_millis = datetime.timestamp_millis();
    match timestamp_millis {
        0 => None,
        _ => Some(timestamp_millis as u64),
    }
}
fn get_unique_symbols(
    fapi_client: &account::FuturesAccount,
    start_t: u64,
    end_t: u64,
) -> BTreeSet<String> {
    let income_req: account::IncomeRequest = account::IncomeRequest {
        symbol: None,
        income_type: None,
        start_time: Some(start_t),
        end_time: Some(end_t),
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
                        pos.time_finished = trades[j].time;
                        // j += 1;
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
