use std::collections::HashMap;

use headset::{EarningsReport, StockQuote};

use crate::portfolio::Portfolio;

pub const TAB_TITLES: &[&str] = &["Indices", "Schwab", "Robinhood"];

pub struct App {
    pub quotes: Vec<StockQuote>,
    pub earnings: Vec<EarningsReport>,
    pub portfolios: Vec<Portfolio>,
    pub portfolio_quotes: HashMap<String, StockQuote>,
    pub should_quit: bool,
    pub loading: bool,
    pub active_tab: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            quotes: Vec::new(),
            earnings: Vec::new(),
            portfolios: Vec::new(),
            portfolio_quotes: HashMap::new(),
            should_quit: false,
            loading: true,
            active_tab: 0,
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % TAB_TITLES.len();
    }

    pub fn prev_tab(&mut self) {
        if self.active_tab == 0 {
            self.active_tab = TAB_TITLES.len() - 1;
        } else {
            self.active_tab -= 1;
        }
    }
}
