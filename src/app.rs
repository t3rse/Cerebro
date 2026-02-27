use std::collections::HashMap;

use headset::{EarningsReport, StockQuote};

use crate::portfolio::Portfolio;

pub const TAB_TITLES: &[&str] = &["Indices", "Schwab", "Robinhood"];

// One focus slot per portfolio tab (indexed by portfolio_idx = active_tab - 1)
const PORTFOLIO_COUNT: usize = 2;

pub struct App {
    pub quotes: Vec<StockQuote>,
    pub earnings: Vec<EarningsReport>,
    pub portfolios: Vec<Portfolio>,
    pub portfolio_quotes: HashMap<String, StockQuote>,
    pub should_quit: bool,
    pub loading: bool,
    pub active_tab: usize,
    pub portfolio_focus: [Option<usize>; PORTFOLIO_COUNT],
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
            portfolio_focus: [None; PORTFOLIO_COUNT],
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

    fn portfolio_idx(&self) -> Option<usize> {
        if self.active_tab == 0 { None } else { Some(self.active_tab - 1) }
    }

    pub fn focus_next(&mut self) {
        let Some(idx) = self.portfolio_idx() else { return };
        let len = self.portfolios[idx].positions.len();
        if len == 0 { return; }
        self.portfolio_focus[idx] = Some(match self.portfolio_focus[idx] {
            None => 0,
            Some(i) => (i + 1).min(len - 1),
        });
    }

    pub fn focus_prev(&mut self) {
        let Some(idx) = self.portfolio_idx() else { return };
        self.portfolio_focus[idx] = match self.portfolio_focus[idx] {
            None | Some(0) => None,
            Some(i) => Some(i - 1),
        };
    }

    pub fn focused_ticker(&self) -> Option<&str> {
        let idx = self.portfolio_idx()?;
        let pos_idx = self.portfolio_focus[idx]?;
        self.portfolios.get(idx)?.positions.get(pos_idx).map(|p| p.ticker.as_str())
    }
}
