#[derive(Default, Clone)]
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
