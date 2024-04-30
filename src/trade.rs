#[derive(Default, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: String,
    pub price: f64,
    pub qty: f64,
    pub realized_pnl: f64,
    pub quote_qty: f64,
    pub commission: f64,
    pub time: u64,
}
