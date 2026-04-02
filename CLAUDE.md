# Cerebro

A terminal-based financial dashboard built with Rust and ratatui. Displays a markets overview, portfolio positions, market news, stock research, and an earnings calendar — all from the command line.

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
│   ├── src/
│   │   ├── lib.rs          # Headset client (quote, market_news, company_news, earnings, basic_financials, filings, company_peers)
│   │   ├── models.rs       # Domain models (StockQuote, MarketNews, CompanyNews, EarningsReport, BasicFinancials, FilingEntry)
│   │   └── error.rs        # HeadsetError, Result type alias
│   └── tests/
│       └── integration.rs  # Integration tests (1 unit + 9 network, #[ignore])
├── rapid/                  # Internal library crate: Economic calendar via RapidAPI
│   ├── src/
│   │   ├── lib.rs          # Rapid client (calendar); reads RAPID_API_KEY env var
│   │   ├── models.rs       # EconEvent (all fields Option<>), CalendarResponse
│   │   └── error.rs        # RapidError (MissingApiKey, Http), Result type alias
│   └── tests/
│       └── integration.rs  # Integration tests (1 unit + 4 network, #[ignore])
├── ydata/                  # Internal library crate: Yahoo Finance historical data
│   ├── src/
│   │   ├── lib.rs          # YData client (get_quote_history) + MarketSnapshot impl; wraps yahoo_finance_api
│   │   ├── models.rs       # Domain models (QuoteBar, MarketSnapshot)
│   │   └── error.rs        # YDataError (Yahoo), Result type alias
│   └── tests/
│       └── integration.rs  # Integration tests (1 unit + 7 network, #[ignore])
├── hodl/                   # Internal library crate: Crypto.com Exchange public REST API client
│   ├── src/
│   │   ├── lib.rs          # Hodl client (get_instruments, get_book, get_candlestick, get_trades, get_tickers, get_valuations)
│   │   ├── models.rs       # Domain models (Instrument, OrderBook, BookLevel, Candle, Trade, Ticker, Valuation)
│   │   └── error.rs        # HodlError (Http, Api), Result type alias
│   └── tests/
│       └── integration.rs  # Integration tests (2 unit + 6 network, #[ignore])
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

# Generate and open rustdoc for all library crates
cargo doc --no-deps --open

# Run all tests (unit + doctests; network tests are skipped by default)
cargo test

# Run network integration tests for a specific crate (requires API keys / network)
cargo test -p headset -- --include-ignored
cargo test -p rapid   -- --include-ignored
cargo test -p ydata   -- --include-ignored
cargo test -p hodl    -- --include-ignored
```

## Configuration

- Create a `.env` file in the project root with `FINNHUB_API_KEY=your_key_here` and `RAPID_API_KEY=your_key_here`
- Portfolio data is embedded at compile time from `examples/schwab_portfolio.json` and `examples/robinhood_portfolio.json`
- Portfolio JSON schema: `{ "portfolio_name": string, "positions": [{ "ticker", "quantity", "cost_basis", "purchase_date", "currency?" }] }`

## Key Bindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `← →` | Switch main tab (Markets Overview / Portfolios / News / Research / Calendar) |
| `[ ]` | Markets Overview: switch focused column; Portfolios: switch account; News: switch category; Research: switch sub-tab |
| `↑ ↓` | Navigate rows in the focused column/list |
| `Enter` | Open focused ticker in browser (Yahoo Finance) |
| `o` | Open focused news article URL in browser |
| `r` | Refresh news for current category |
| `:` | Open symbol search (Research tab only) |
| `Esc` | Cancel symbol search input |
| `↑ ↓` | Navigate filings list or peers list (Research tab) |
| `o` | Open focused SEC filing URL in browser (Filings sub-tab) |
| `↑ ↓` | Navigate economic calendar events (Calendar tab) |

## Testing

Each library crate has an integration test file at `<crate>/tests/integration.rs`.  Tests fall into two categories:

- **Unit tests** — run with `cargo test`, no keys or network needed.
- **Network tests** — marked `#[ignore]`; require env vars and a live connection.  Run with `--include-ignored`.

| Crate | Unit tests | Network tests (ignored) |
|-------|-----------|------------------------|
| `headset` | `missing_api_key_error_display` | `quote`, `market_news`, `market_news_pagination`, `company_news`, `basic_financials`, `filings`, `company_peers`, `earnings` (×2) |
| `rapid` | `missing_api_key_error_display` | `calendar_us_with_date_range`, `calendar_no_filter`, `calendar_event_has_title_and_date`, `calendar_importance_in_known_range` |
| `ydata` | `result_alias_ok_works` | `get_quote_history_returns_bars`, `get_quote_history_bars_are_chronological`, `get_quote_history_adjclose_near_close`, `get_quote_history_invalid_ticker_returns_error`, `market_snapshot_new_contains_all_tickers`, `market_snapshot_fetch_explicit_range`, `market_snapshot_invalid_ticker_returns_error` |
| `hodl` | `result_alias_ok_works`, `error_display_api` | `get_instruments_returns_data`, `get_book_returns_bids_and_asks`, `get_candlestick_returns_candles`, `get_trades_returns_trades`, `get_tickers_all_returns_many`, `get_tickers_single_returns_one`, `get_valuations_index_price` |

Doctests in each `lib.rs` are compiled and run as `no_run` examples via `cargo test`.

## Architecture Notes

- **`headset` crate** is a thin wrapper around the `finnhub` crate, mapping its types to clean domain models. All API calls go through `Headset`. Methods: `quote`, `market_news`, `company_news`, `earnings`, `basic_financials`, `filings`, `company_peers`.
- **`ydata` crate** wraps `yahoo_finance_api`. `YData::new()` creates a client. Methods: `get_quote_history(ticker, start, end) -> Vec<QuoteBar>`. Also exports `MarketSnapshot` — see below. No API key required.
- **`MarketSnapshot`** (in `ydata`) holds a `HashMap<String, Vec<QuoteBar>>` keyed by ticker, plus the `start`/`end` window. Constructed via `MarketSnapshot::new(&client, tickers)` (defaults to today's 09:30 ET → now) or `MarketSnapshot::fetch(&client, tickers, start, end)` for an explicit range.
- **`rapid` crate** wraps the Ultimate Economic Calendar RapidAPI (`ultimate-economic-calendar.p.rapidapi.com`). The `Rapid` client reads `RAPID_API_KEY` from the environment. Single method: `calendar(country: Option<&str>, from: Option<&str>, to: Option<&str>) -> Vec<EconEvent>`. Country is appended as a path segment; date range passed as `from`/`to` query params (`YYYY-MM-DD`). `EconEvent` fields are all `Option<>` to handle sparse API responses.
- **`hodl` crate** wraps the Crypto.com Exchange public REST API (`api.crypto.com/exchange/v1`). No API key required. `Hodl::new()` creates a client. Methods: `get_instruments()`, `get_book(instrument_name, depth)`, `get_candlestick(instrument_name, timeframe, count)`, `get_trades(instrument_name, count)`, `get_tickers(instrument_name)`, `get_valuations(instrument_name, valuation_type, count)`.
- **`App`** holds all application state. Navigation state (`main_tab`, `active_tab`, `news_tab`, `markets_col`, `markets_row`, focus indices) lives here alongside data (`quotes`, `news_articles`, `portfolios`, `market_snapshots`, etc.).
- **`ui.rs`** is purely a rendering layer — it reads `&App` and produces ratatui widgets. No state mutation happens here.
- **`main.rs`** owns the async runtime and event loop. API fetches happen upfront before the TUI starts (blocking on load), then the loop handles key events and render ticks.
- The app uses two timers via `EventHandler`: a tick timer (250ms) and a render timer (33ms ≈ 30fps).

## Markets Overview Tab

The first tab (index 0) displays a multi-column dashboard of market data loaded at startup from Yahoo Finance via the `ydata` crate. Three `MarketSnapshot` objects are fetched over a 30-day window and stored in `App::market_snapshots` (a `BTreeMap<String, MarketSnapshot>`):

| Key | Tickers |
|-----|---------|
| `Commodities` | GC=F, SI=F, CL=F, NG=F, HG=F, ZC=F, ZS=F, ZB=F, ZN=F, ZM=F, ZT=F |
| `Indices` | ^DJI, ^GSPC, ^RUT |
| `Sectors` | XLK, XLV, XLF, XLY, XLP, XLE, XLI, XLB, XLU, XLRE, XLC |

Each column renders as a `Table` with columns: Ticker, Close, Chg (30d), Chg % (30d). Change is computed as `latest.close − first.open`. Green = positive, red = negative.

The active column is highlighted with a yellow border. A 12-row line chart at the bottom tracks the close price history of the currently focused ticker over the full 30-day window.

Navigation: `[ ]` switches columns, `↑ ↓` moves the row cursor within the active column.

## Main Tabs

| Index | Name | Sub-tabs / Notes |
|-------|------|-----------------|
| 0 | Markets Overview | `[ ]` column, `↑ ↓` row, line chart updates on focus |
| 1 | Portfolios | Indices / Schwab / Robinhood sub-tabs |
| 2 | News | General / Forex / Crypto / Merger sub-tabs |
| 3 | Research | Basic Financials / Filings / Company Peers sub-tabs |
| 4 | Calendar | Economic events list + detail pane |

## Dependencies

- `ratatui` — TUI framework
- `crossterm` — terminal backend and key event stream
- `tokio` — async runtime (full features)
- `time` — date/time handling (timestamp → calendar date in chart labels)
- `headset` (internal) → `finnhub` — financial data API
- `rapid` (internal) → RapidAPI Ultimate Economic Calendar — economic events
- `ydata` (internal) → `yahoo_finance_api` — historical OHLCV data + `MarketSnapshot`
- `hodl` (internal) → Crypto.com Exchange REST API — crypto market data (no key required)
- `serde` / `serde_json` — portfolio JSON parsing
- `dotenvy` — `.env` file loading
