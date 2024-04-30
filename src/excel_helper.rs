extern crate simple_excel_writer;
use crate::position::Position;
use chrono::{FixedOffset, NaiveDateTime, Utc};
use excel::{row, Workbook};
use simple_excel_writer as excel;
use simple_excel_writer::Row;

pub fn write_to_excel(positions: &mut Vec<Position>) {
    let ukraine_timezone = FixedOffset::east_opt(3 * 3600).unwrap();

    let current_time = Utc::now().with_timezone(&ukraine_timezone);

    positions.sort_by(|a, b| b.time_start.cmp(&a.time_start));

    let mut wb = Workbook::create(
        format!(
            "output_positions_{}.xlsx",
            current_time.format("%Y-%m-%d_%H-%M-%S")
        )
        .as_str(),
    );
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
