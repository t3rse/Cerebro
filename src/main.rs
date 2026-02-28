mod app;
mod event;
mod portfolio;
mod ui;

use std::collections::HashMap;
use std::time::Duration;

use crossterm::event::KeyCode;
use headset::Headset;

use app::App;
use event::{Event, EventHandler};

const SCHWAB_JSON: &str = include_str!("../examples/schwab_portfolio.json");
const ROBINHOOD_JSON: &str = include_str!("../examples/robinhood_portfolio.json");

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = match Headset::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create Headset client: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new();

    // Load portfolios from embedded JSON
    let schwab: portfolio::Portfolio = serde_json::from_str(SCHWAB_JSON)
        .expect("Failed to parse schwab_portfolio.json");
    let robinhood: portfolio::Portfolio = serde_json::from_str(ROBINHOOD_JSON)
        .expect("Failed to parse robinhood_portfolio.json");

    // Collect all portfolio tickers for quote fetching
    let portfolio_tickers: Vec<String> = schwab
        .positions
        .iter()
        .chain(robinhood.positions.iter())
        .map(|p| p.ticker.clone())
        .collect();

    app.portfolios = vec![schwab, robinhood];

    // Fetch index quotes
    for symbol in ["DIA", "SPY", "NDAQ"] {
        if let Ok(q) = client.quote(symbol).await {
            app.quotes.push(q);
        }
    }

    // Fetch earnings calendar
    if let Ok(reports) = client.earnings(None, None, None).await {
        app.earnings = reports;
    }

    // Fetch portfolio quotes
    let mut portfolio_quotes = HashMap::new();
    for ticker in &portfolio_tickers {
        if let Ok(q) = client.quote(ticker).await {
            portfolio_quotes.insert(ticker.clone(), q);
        }
    }
    app.portfolio_quotes = portfolio_quotes;

    app.loading = false;

    let mut terminal = ratatui::init();
    let mut events = EventHandler::new(Duration::from_millis(250), Duration::from_millis(33));

    while !app.should_quit {
        match events.next().await {
            Some(Event::Key(key)) => match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Right => app.next_main_tab(),
                KeyCode::Left => app.prev_main_tab(),
                KeyCode::Char(']') => app.next_tab(),
                KeyCode::Char('[') => app.prev_tab(),
                KeyCode::Down => app.focus_next(),
                KeyCode::Up => app.focus_prev(),
                KeyCode::Enter => {
                    if let Some(ticker) = app.focused_ticker() {
                        let url = format!("https://finance.yahoo.com/quote/{}/", ticker);
                        std::process::Command::new("open").arg(&url).spawn().ok();
                    }
                }
                _ => {}
            },
            Some(Event::Render) => {
                terminal
                    .draw(|frame| ui::render(frame, &app))
                    .expect("failed to draw frame");
            }
            Some(Event::Tick) => {}
            None => break,
        }
    }

    ratatui::restore();
}
