use crate::trade::Trade;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use binance::api::*;
use binance::errors::Error;
use binance::futures::*;
use std::collections::{BTreeSet, HashSet};
use serde_json::Value;

pub fn get_timestamp_mil2(period: &str) -> Vec<(u64, u64)> {
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
                println!(
                    "start_of_day: {:?}  end_of_day: {:?}",
                    start_of_day.clone(),
                    end_of_day.clone()
                );
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

pub fn get_timestamp_mil(date_str: &str) -> Option<u64> {
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

pub fn get_unique_symbols(
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

pub fn get_symbol_trades(
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
