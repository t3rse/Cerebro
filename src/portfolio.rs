use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Portfolio {
    pub portfolio_name: String,
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    pub ticker: String,
    pub quantity: f64,
    pub cost_basis: f64,
    pub purchase_date: String,
    pub currency: Option<String>,
}
