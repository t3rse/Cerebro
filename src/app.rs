use std::collections::HashMap;

use headset::{CompanyNews, EarningsReport, MarketNews, StockQuote};

use crate::portfolio::Portfolio;

pub const MAIN_TABS: &[&str] = &["Portfolios", "News", "Research", "Calendar"];
pub const TAB_TITLES: &[&str] = &["Indices", "Schwab", "Robinhood"];
pub const NEWS_TABS: &[&str] = &["General", "Forex", "Crypto", "Merger"];

const PORTFOLIO_COUNT: usize = 2;

pub enum ResearchMode {
    Idle,
    Inputting(String),
}

pub struct App {
    pub quotes: Vec<StockQuote>,
    pub earnings: Vec<EarningsReport>,
    pub portfolios: Vec<Portfolio>,
    pub portfolio_quotes: HashMap<String, StockQuote>,
    pub news_articles: Vec<Vec<MarketNews>>,
    // Research
    pub research_mode: ResearchMode,
    pub research_symbol: Option<String>,
    pub research_quote: Option<StockQuote>,
    pub research_news: Vec<CompanyNews>,
    pub research_news_focus: usize,
    // Navigation
    pub should_quit: bool,
    pub loading: bool,
    pub main_tab: usize,
    pub active_tab: usize,
    pub news_tab: usize,
    pub news_focus: usize,
    pub portfolio_focus: [Option<usize>; PORTFOLIO_COUNT],
}

impl App {
    pub fn new() -> Self {
        Self {
            quotes: Vec::new(),
            earnings: Vec::new(),
            portfolios: Vec::new(),
            portfolio_quotes: HashMap::new(),
            news_articles: vec![Vec::new(); NEWS_TABS.len()],
            research_mode: ResearchMode::Idle,
            research_symbol: None,
            research_quote: None,
            research_news: Vec::new(),
            research_news_focus: 0,
            should_quit: false,
            loading: true,
            main_tab: 0,
            active_tab: 0,
            news_tab: 0,
            news_focus: 0,
            portfolio_focus: [None; PORTFOLIO_COUNT],
        }
    }

    // ── Research ──────────────────────────────────────────────────────────────

    pub fn is_research_inputting(&self) -> bool {
        matches!(self.research_mode, ResearchMode::Inputting(_))
    }

    pub fn research_start_input(&mut self) {
        self.research_mode = ResearchMode::Inputting(String::new());
    }

    pub fn research_input_push(&mut self, c: char) {
        if let ResearchMode::Inputting(ref mut s) = self.research_mode {
            s.push(c);
        }
    }

    pub fn research_input_pop(&mut self) {
        if let ResearchMode::Inputting(ref mut s) = self.research_mode {
            s.pop();
        }
    }

    pub fn research_input_text(&self) -> &str {
        match &self.research_mode {
            ResearchMode::Inputting(s) => s.as_str(),
            ResearchMode::Idle => "",
        }
    }

    pub fn research_cancel(&mut self) {
        self.research_mode = ResearchMode::Idle;
    }

    /// Consumes the input, stores the uppercased symbol, and returns it for fetching.
    /// Returns `None` if the input was empty.
    pub fn research_submit(&mut self) -> Option<String> {
        let symbol = match &self.research_mode {
            ResearchMode::Inputting(s) => s.trim().to_uppercase(),
            ResearchMode::Idle => return None,
        };
        self.research_mode = ResearchMode::Idle;
        if symbol.is_empty() {
            return None;
        }
        self.research_symbol = Some(symbol.clone());
        Some(symbol)
    }

    pub fn research_news_focus_next(&mut self) {
        let len = self.research_news.len();
        if len == 0 {
            return;
        }
        self.research_news_focus = (self.research_news_focus + 1).min(len - 1);
    }

    pub fn research_news_focus_prev(&mut self) {
        self.research_news_focus = self.research_news_focus.saturating_sub(1);
    }

    pub fn focused_research_news_url(&self) -> Option<&str> {
        self.research_news
            .get(self.research_news_focus)
            .map(|a| a.url.as_str())
    }

    // ── Main nav ──────────────────────────────────────────────────────────────

    pub fn next_main_tab(&mut self) {
        self.main_tab = (self.main_tab + 1) % MAIN_TABS.len();
    }

    pub fn prev_main_tab(&mut self) {
        if self.main_tab == 0 {
            self.main_tab = MAIN_TABS.len() - 1;
        } else {
            self.main_tab -= 1;
        }
    }

    // ── Portfolio sub-tabs ────────────────────────────────────────────────────

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

    // ── News sub-tabs ─────────────────────────────────────────────────────────

    pub fn next_news_tab(&mut self) {
        self.news_tab = (self.news_tab + 1) % NEWS_TABS.len();
        self.news_focus = 0;
    }

    pub fn prev_news_tab(&mut self) {
        if self.news_tab == 0 {
            self.news_tab = NEWS_TABS.len() - 1;
        } else {
            self.news_tab -= 1;
        }
        self.news_focus = 0;
    }

    pub fn news_focus_next(&mut self) {
        let len = self.news_articles[self.news_tab].len();
        if len == 0 {
            return;
        }
        self.news_focus = (self.news_focus + 1).min(len - 1);
    }

    pub fn news_focus_prev(&mut self) {
        self.news_focus = self.news_focus.saturating_sub(1);
    }

    pub fn focused_news_url(&self) -> Option<&str> {
        self.news_articles
            .get(self.news_tab)?
            .get(self.news_focus)
            .map(|a| a.url.as_str())
    }

    // ── Portfolio position focus ──────────────────────────────────────────────

    fn portfolio_idx(&self) -> Option<usize> {
        if self.main_tab != 0 || self.active_tab == 0 {
            None
        } else {
            Some(self.active_tab - 1)
        }
    }

    pub fn focus_next(&mut self) {
        let Some(idx) = self.portfolio_idx() else {
            return;
        };
        let len = self.portfolios[idx].positions.len();
        if len == 0 {
            return;
        }
        self.portfolio_focus[idx] = Some(match self.portfolio_focus[idx] {
            None => 0,
            Some(i) => (i + 1).min(len - 1),
        });
    }

    pub fn focus_prev(&mut self) {
        let Some(idx) = self.portfolio_idx() else {
            return;
        };
        self.portfolio_focus[idx] = match self.portfolio_focus[idx] {
            None | Some(0) => None,
            Some(i) => Some(i - 1),
        };
    }

    pub fn focused_ticker(&self) -> Option<&str> {
        let idx = self.portfolio_idx()?;
        let pos_idx = self.portfolio_focus[idx]?;
        self.portfolios
            .get(idx)?
            .positions
            .get(pos_idx)
            .map(|p| p.ticker.as_str())
    }
}
