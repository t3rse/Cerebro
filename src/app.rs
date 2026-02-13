use headset::{EarningsReport, StockQuote};

pub struct App {
    pub quote: Option<StockQuote>,
    pub earnings: Vec<EarningsReport>,
    pub should_quit: bool,
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            quote: None,
            earnings: Vec::new(),
            should_quit: false,
            loading: true,
        }
    }
}
