use headset::{EarningsReport, StockQuote};

pub struct App {
    pub quotes: Vec<StockQuote>,
    pub earnings: Vec<EarningsReport>,
    pub should_quit: bool,
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            quotes: Vec::new(),
            earnings: Vec::new(),
            should_quit: false,
            loading: true,
        }
    }
}
