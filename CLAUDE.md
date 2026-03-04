# Cerebro

A terminal-based financial dashboard built with Rust and ratatui. Displays portfolio positions, market news, stock research, and an earnings calendar — all from the command line.

## Project Structure

```
Cerebro/
├── src/                    # Main binary (TUI application)
│   ├── main.rs             # Entry point, event loop, key bindings, API calls
│   ├── app.rs              # Application state (App struct, navigation logic)
│   ├── ui.rs               # All ratatui rendering logic
│   ├── event.rs            # Async event handler (key events + render/tick timers)
│   └── portfolio.rs        # Portfolio/Position data structures (serde Deserialize)
├── headset/                # Internal library crate: Finnhub API client wrapper
│   └── src/
│       ├── lib.rs          # Headset client (quote, market_news, company_news, earnings)
│       ├── models.rs       # Domain models (StockQuote, MarketNews, CompanyNews, EarningsReport)
│       └── error.rs        # HeadsetError, Result type alias
└── examples/               # Sample portfolio JSON files embedded at compile time
    ├── schwab_portfolio.json
    └── robinhood_portfolio.json
```

## Build & Run

```bash
# Requires FINNHUB_API_KEY in environment or .env file
cargo run

# Build release binary
cargo build --release
```

## Configuration

- Create a `.env` file in the project root with `FINNHUB_API_KEY=your_key_here`
- Portfolio data is embedded at compile time from `examples/schwab_portfolio.json` and `examples/robinhood_portfolio.json`
- Portfolio JSON schema: `{ "portfolio_name": string, "positions": [{ "ticker", "quantity", "cost_basis", "purchase_date", "currency?" }] }`

## Key Bindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `← →` | Switch main tab (Portfolios / News / Research / Calendar) |
| `[ ]` | Switch sub-tab (portfolio accounts or news categories) |
| `↑ ↓` | Navigate list items |
| `Enter` | Open focused ticker in browser (Yahoo Finance) |
| `o` | Open focused news article URL in browser |
| `r` | Refresh news for current category |
| `:` | Open symbol search (Research tab only) |
| `Esc` | Cancel symbol search input |
| `[ ]` | Switch research sub-tab (Basic Financials / Filings / Company Peers) |
| `↑ ↓` | Navigate filings list or peers list (Research tab) |
| `o` | Open focused SEC filing URL in browser (Filings sub-tab) |

## Architecture Notes

- **`headset` crate** is a thin wrapper around the `finnhub` crate, mapping its types to clean domain models. All API calls go through `Headset`. Methods: `quote`, `market_news`, `company_news`, `earnings`, `basic_financials`, `filings`, `company_peers`.
- **`App`** holds all application state. Navigation state (`main_tab`, `active_tab`, `news_tab`, focus indices) lives here alongside data (`quotes`, `news_articles`, `portfolios`, etc.).
- **`ui.rs`** is purely a rendering layer — it reads `&App` and produces ratatui widgets. No state mutation happens here.
- **`main.rs`** owns the async runtime and event loop. API fetches happen upfront before the TUI starts (blocking on load), then the loop handles key events and render ticks.
- The app uses two timers via `EventHandler`: a tick timer (250ms) and a render timer (33ms ≈ 30fps).

## Dependencies

- `ratatui` — TUI framework
- `crossterm` — terminal backend and key event stream
- `tokio` — async runtime (full features)
- `headset` (internal) → `finnhub` — financial data API
- `serde` / `serde_json` — portfolio JSON parsing
- `dotenvy` — `.env` file loading
