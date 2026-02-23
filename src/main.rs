mod app;
mod event;
mod ui;

use std::time::Duration;

use crossterm::event::KeyCode;
use headset::Headset;

use app::App;
use event::{Event, EventHandler};

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

    // Fetch data before entering the TUI
    for symbol in ["DIA", "SPY", "NDAQ"] {
        if let Ok(q) = client.quote(symbol).await {
            app.quotes.push(q);
        }
    }
    if let Ok(reports) = client.earnings(None, None, None).await {
        app.earnings = reports;
    }
    app.loading = false;

    let mut terminal = ratatui::init();
    let mut events = EventHandler::new(Duration::from_millis(250), Duration::from_millis(33));

    while !app.should_quit {
        match events.next().await {
            Some(Event::Key(key)) => {
                if key.code == KeyCode::Char('q') {
                    app.should_quit = true;
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
