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

impl Position {
    fn make_positions(trades: &mut Vec<Trade>) -> Vec<Position> {
        let mut positions = Vec::new();
        let mut i = 0;

        while i < trades.len() {
            let mut pos = Position::default();
            pos.symbol = trades[i].symbol.clone();
            pos.side = trades[i].side.clone();
            pos.time_start = trades[i].time;

            let mut j = i;
            while j < trades.len() && pos.side == trades[j].side {
                pos.average_entry_price += trades[j].price * trades[j].qty;
                pos.volume_quantity += trades[j].qty;
                pos.volume_dollar += trades[j].quote_qty;
                pos.commission += trades[j].commission;
                j += 1;
            }

            let mut exit_volume_quantity = 0.0;
            let mut realized_pnl_net = 0.0;
            let mut average_exit_price = 0.0;
            let mut exit_commission = 0.0;

            while j < trades.len() && pos.side != trades[j].side {
                average_exit_price += trades[j].price * trades[j].qty;
                exit_volume_quantity += trades[j].qty;
                realized_pnl_net += trades[j].realized_pnl;
                exit_commission += trades[j].commission;
                j += 1;
            }

            if (pos.volume_quantity - exit_volume_quantity).abs() < 0.000001 {
                pos.average_entry_price /= pos.volume_quantity;
                pos.average_exit_price = average_exit_price / exit_volume_quantity;
                pos.realized_pnl_net = realized_pnl_net;
                pos.realized_pnl_gross = realized_pnl_net - pos.commission - exit_commission;
                pos.exit_volume_quantity = exit_volume_quantity;
                pos.time_finished = trades[j - 1].time;
                i = j;
            } else {
                i += 1;
                continue;
            }

            positions.push(pos);
        }

        positions
    }
}
