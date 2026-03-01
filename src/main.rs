mod app;
mod event;
mod portfolio;
mod ui;

use std::collections::HashMap;
use std::time::Duration;

use crossterm::event::KeyCode;
use headset::{Headset, NewsCategory};

use app::App;
use event::{Event, EventHandler};

const SCHWAB_JSON: &str = include_str!("../examples/schwab_portfolio.json");
const ROBINHOOD_JSON: &str = include_str!("../examples/robinhood_portfolio.json");

fn news_tab_to_category(tab: usize) -> NewsCategory {
    match tab {
        0 => NewsCategory::General,
        1 => NewsCategory::Forex,
        2 => NewsCategory::Crypto,
        _ => NewsCategory::Merger,
    }
}

/// Returns a `YYYY-MM-DD` string for `days_ago` days before today.
fn date_days_ago(days_ago: u64) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(days_ago * 86400);

    let mut remaining = secs / 86400;
    let mut year = 1970u64;
    loop {
        let y_days = if is_leap(year) { 366 } else { 365 };
        if remaining < y_days {
            break;
        }
        remaining -= y_days;
        year += 1;
    }
    let month_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        month += 1;
    }
    format!("{year:04}-{month:02}-{:02}", remaining + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

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
    let schwab: portfolio::Portfolio =
        serde_json::from_str(SCHWAB_JSON).expect("Failed to parse schwab_portfolio.json");
    let robinhood: portfolio::Portfolio =
        serde_json::from_str(ROBINHOOD_JSON).expect("Failed to parse robinhood_portfolio.json");

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

    // Fetch market news for each category (order matches NEWS_TABS)
    for i in 0..4 {
        if let Ok(articles) = client.market_news(news_tab_to_category(i), None).await {
            app.news_articles[i] = articles;
        }
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
            Some(Event::Key(key)) => {
                // Research command palette intercepts all keys while open.
                if app.is_research_inputting() {
                    match key.code {
                        KeyCode::Esc => app.research_cancel(),
                        KeyCode::Backspace => app.research_input_pop(),
                        KeyCode::Enter => {
                            if let Some(symbol) = app.research_submit() {
                                let today = date_days_ago(0);
                                let from = date_days_ago(30);
                                app.research_quote = client.quote(&symbol).await.ok();
                                app.research_news = client
                                    .company_news(&symbol, &from, &today)
                                    .await
                                    .unwrap_or_default();
                                app.research_news_focus = 0;
                            }
                        }
                        KeyCode::Char(c) => app.research_input_push(c.to_ascii_uppercase()),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Right => app.next_main_tab(),
                        KeyCode::Left => app.prev_main_tab(),
                        KeyCode::Char(':') if app.main_tab == 2 => app.research_start_input(),
                        KeyCode::Char(']') => match app.main_tab {
                            0 => app.next_tab(),
                            1 => app.next_news_tab(),
                            _ => {}
                        },
                        KeyCode::Char('[') => match app.main_tab {
                            0 => app.prev_tab(),
                            1 => app.prev_news_tab(),
                            _ => {}
                        },
                        KeyCode::Down => match app.main_tab {
                            0 => app.focus_next(),
                            1 => app.news_focus_next(),
                            2 => app.research_news_focus_next(),
                            _ => {}
                        },
                        KeyCode::Up => match app.main_tab {
                            0 => app.focus_prev(),
                            1 => app.news_focus_prev(),
                            2 => app.research_news_focus_prev(),
                            _ => {}
                        },
                        KeyCode::Enter => {
                            if let Some(ticker) = app.focused_ticker() {
                                let url = format!("https://finance.yahoo.com/quote/{}/", ticker);
                                std::process::Command::new("open").arg(&url).spawn().ok();
                            }
                        }
                        KeyCode::Char('o') if app.main_tab == 1 => {
                            if let Some(url) = app.focused_news_url() {
                                std::process::Command::new("open").arg(url).spawn().ok();
                            }
                        }
                        KeyCode::Char('o') if app.main_tab == 2 => {
                            if let Some(url) = app.focused_research_news_url() {
                                std::process::Command::new("open").arg(url).spawn().ok();
                            }
                        }
                        KeyCode::Char('r') if app.main_tab == 1 => {
                            let tab = app.news_tab;
                            if let Ok(articles) =
                                client.market_news(news_tab_to_category(tab), None).await
                            {
                                app.news_articles[tab] = articles;
                                app.news_focus = 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
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
