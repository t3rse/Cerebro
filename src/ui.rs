use crate::app::{App, MAIN_TABS, NEWS_TABS, RESEARCH_SUB_TABS, TAB_TITLES};
use ratatui::text::Text;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Dataset, GraphType, List, ListItem, ListState,
        Paragraph, Row, Table, TableState, Tabs, Wrap,
    },
};
use time::OffsetDateTime;
use ydata::{MarketSnapshot, QuoteBar};

pub fn render(frame: &mut Frame, app: &App) {
    // Portfolios (1) and News (2) both have a sub-tabs row.
    let has_sub_tabs = app.main_tab == 1 || app.main_tab == 2;

    let constraints: Vec<Constraint> = if has_sub_tabs {
        vec![
            Constraint::Length(3), // title
            Constraint::Length(3), // main nav
            Constraint::Length(3), // section sub-tabs
            Constraint::Fill(1),   // content
            Constraint::Length(3), // status bar
        ]
    } else {
        vec![
            Constraint::Length(3), // title
            Constraint::Length(3), // main nav
            Constraint::Fill(1),   // content
            Constraint::Length(3), // status bar
        ]
    };

    let outer = Layout::vertical(constraints).split(frame.area());

    render_title(frame, outer[0]);
    render_main_nav(frame, app, outer[1]);

    let (content_area, status_area) = if has_sub_tabs {
        match app.main_tab {
            1 => render_portfolio_sub_tabs(frame, app, outer[2]),
            2 => render_news_sub_tabs(frame, app, outer[2]),
            _ => {}
        }
        (outer[3], outer[4])
    } else {
        (outer[2], outer[3])
    };

    if app.loading {
        frame.render_widget(
            Paragraph::new("Loading data...").block(Block::default().borders(Borders::ALL)),
            content_area,
        );
    } else {
        match app.main_tab {
            0 => render_markets_overview_tab(frame, app, content_area),
            1 => match app.active_tab {
                0 => render_indices_tab(frame, app, content_area),
                1 => render_portfolio_tab(frame, app, content_area, 0),
                2 => render_portfolio_tab(frame, app, content_area, 1),
                _ => {}
            },
            2 => render_news_tab(frame, app, content_area),
            3 => render_research_tab(frame, app, content_area),
            4 => render_calendar_tab(frame, app, content_area),
            _ => render_placeholder(frame, app, content_area),
        }
    }

    render_status_bar(frame, app, status_area);
}

fn render_main_nav(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tabs = Tabs::new(MAIN_TABS.iter().copied())
        .select(app.main_tab)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(tabs, area);
}

fn render_portfolio_sub_tabs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tabs = Tabs::new(TAB_TITLES.iter().copied())
        .select(app.active_tab)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(tabs, area);
}

fn render_news_sub_tabs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tabs = Tabs::new(NEWS_TABS.iter().copied())
        .select(app.news_tab)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(tabs, area);
}

fn render_news_tab(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let panes =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).split(area);

    render_news_list(frame, app, panes[0]);
    render_news_detail(frame, app, panes[1]);
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", truncated.trim_end())
    }
}

fn render_news_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let articles = &app.news_articles[app.news_tab];
    let category_label = NEWS_TABS[app.news_tab];

    let inner_width = area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = articles
        .iter()
        .map(|article| {
            let headline = truncate(&article.headline, inner_width);
            let meta = format!("{} · {}", article.source, relative_time(article.datetime));
            let meta_truncated = truncate(&meta, inner_width);

            // Show category badge only for non-"top news" categories
            let category_badge = if article.category != "top news" {
                format!(" [{}]", article.category)
            } else {
                String::new()
            };

            let line1 = Line::from(vec![
                Span::styled(headline, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(category_badge, Style::default().fg(Color::Yellow)),
            ]);
            let line2 = Line::from(Span::styled(
                meta_truncated,
                Style::default().fg(Color::DarkGray),
            ));

            ListItem::new(Text::from(vec![line1, line2, Line::from("")]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" {category_label} "))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !articles.is_empty() {
        state.select(Some(app.news_focus));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_news_detail(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let articles = &app.news_articles[app.news_tab];

    if articles.is_empty() {
        frame.render_widget(
            Paragraph::new("No articles available.").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let a = &articles[app.news_focus.min(articles.len() - 1)];
    render_article_detail(
        frame,
        area,
        &a.headline,
        &a.source,
        a.datetime,
        &a.category,
        &a.summary,
        &a.url,
    );
}

/// Shared detail pane renderer used by both News and Research.
#[allow(clippy::too_many_arguments)]
fn render_article_detail(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    headline: &str,
    source: &str,
    datetime: i64,
    category: &str,
    summary: &str,
    url: &str,
) {
    let rel_time = relative_time(datetime);
    let meta = format!("  {} • {} • {}", source, rel_time, category);
    let sep = "─".repeat(area.width.saturating_sub(2) as usize);
    let url_display = strip_scheme(url);
    let summary_width = area.width.saturating_sub(4) as usize;
    let wrapped_summary = word_wrap(summary, summary_width);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(Span::styled(
            headline.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            meta,
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::raw(sep),
    ];

    for s in wrapped_summary {
        lines.push(Line::raw(format!("  {s}")));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[o]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" open  {url_display}")),
    ]));

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

// ── Research tab ──────────────────────────────────────────────────────────────

fn render_research_tab(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // If the command palette is open, carve off a 3-row input bar at the top.
    let content_area = if app.is_research_inputting() {
        let split = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);
        render_research_input(frame, app, split[0]);
        split[1]
    } else {
        area
    };

    if app.research_quote.is_some() {
        let rows = Layout::vertical([
            Constraint::Length(6), // quote pane
            Constraint::Length(3), // sub-tabs bar
            Constraint::Fill(1),   // sub-tab content
        ])
        .split(content_area);
        render_research_quote(frame, app, rows[0]);
        render_research_sub_tabs(frame, app, rows[1]);
        match app.research_sub_tab {
            0 => render_basic_financials(frame, app, rows[2]),
            1 => render_filings(frame, app, rows[2]),
            2 => render_company_peers(frame, app, rows[2]),
            _ => {}
        }
    } else {
        render_research_empty(frame, content_area);
    }
}

fn render_research_sub_tabs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tabs = Tabs::new(RESEARCH_SUB_TABS.iter().copied())
        .select(app.research_sub_tab)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(tabs, area);
}

fn render_basic_financials(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let Some(ref bf) = app.research_financials else {
        frame.render_widget(
            Paragraph::new("No financial data available.")
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };

    const METRICS: &[(&str, &str)] = &[
        ("52WeekHigh", "52-Week High"),
        ("52WeekLow", "52-Week Low"),
        ("marketCapitalization", "Market Cap (M)"),
        ("peBasicExclExtraTTM", "P/E (TTM)"),
        ("pbAnnual", "P/B"),
        ("dividendYieldIndicatedAnnual", "Dividend Yield"),
        ("epsBasicExclExtraItemsTTM", "EPS (TTM)"),
        ("beta", "Beta"),
        ("currentRatioAnnual", "Current Ratio"),
        ("debtToEquityAnnual", "Debt/Equity"),
        ("revenueGrowth3Y", "Rev Growth 3Y"),
        ("epsGrowth3Y", "EPS Growth 3Y"),
        ("roaTTM", "ROA (TTM)"),
        ("roeTTM", "ROE (TTM)"),
    ];

    let rows: Vec<Row> = METRICS
        .iter()
        .filter_map(|(key, label)| {
            let val = bf.metrics.get(*key)?;
            let display = if let Some(f) = val.as_f64() {
                format!("{f:.2}")
            } else if let Some(s) = val.as_str() {
                s.to_string()
            } else {
                return None;
            };
            Some(Row::new(vec![
                Cell::from(*label).style(Style::default().fg(Color::Yellow)),
                Cell::from(display),
            ]))
        })
        .collect();

    let widths = [Constraint::Percentage(40), Constraint::Percentage(60)];
    let table = Table::new(rows, widths).block(
        Block::default()
            .title(" Basic Financials ")
            .borders(Borders::ALL),
    );
    frame.render_widget(table, area);
}

fn render_filings(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let filings = &app.research_filings;

    let items: Vec<ListItem> = filings
        .iter()
        .map(|f| {
            let form = f.form.as_deref().unwrap_or("—");
            let date = f.filed_date.as_deref().unwrap_or("—");
            let url = f
                .report_url
                .as_deref()
                .or(f.filing_url.as_deref())
                .unwrap_or("");
            let url_display = url
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            ListItem::new(Text::from(vec![
                Line::from(vec![
                    Span::styled(
                        form.to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("   "),
                    Span::raw(date.to_string()),
                ]),
                Line::from(Span::styled(
                    url_display.to_string(),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::raw(""),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" SEC Filings ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !filings.is_empty() {
        state.select(Some(app.research_filings_focus));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_company_peers(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let peers = &app.research_peers;

    let items: Vec<ListItem> = peers
        .iter()
        .map(|symbol| {
            ListItem::new(Line::from(Span::styled(
                symbol.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Company Peers ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !peers.is_empty() {
        state.select(Some(app.research_peers_focus));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_research_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let text = format!(": {}▌", app.research_input_text());
    let p = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Symbol Search ")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(p, area);
}

fn render_research_empty(frame: &mut Frame, area: ratatui::layout::Rect) {
    let p = Paragraph::new(vec![
        Line::raw(""),
        Line::from(Span::styled(
            "  Press : to search for a symbol",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn render_research_quote(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let Some(q) = &app.research_quote else { return };
    let change_color = if q.change >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };
    let sign = if q.change >= 0.0 { "+" } else { "" };

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("${:.2}", q.current_price),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                format!(
                    "{}{:.2}  ({}{:.2}%)",
                    sign, q.change, sign, q.percent_change
                ),
                Style::default().fg(change_color),
            ),
        ]),
        Line::from(Span::styled(
            format!(
                "  H ${:.2}  L ${:.2}  O ${:.2}  C ${:.2}",
                q.high, q.low, q.open, q.previous_close
            ),
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let symbol = app.research_symbol.as_deref().unwrap_or("");
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(format!(" {symbol} "))
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn relative_time(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = (now - ts).max(0);
    if diff < 60 {
        format!("{diff}s ago")
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn strip_scheme(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string()
}

fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// ── Calendar tab ──────────────────────────────────────────────────────────────

fn render_calendar_tab(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let panes =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    render_calendar_list(frame, app, panes[0]);
    render_calendar_detail(frame, app, panes[1]);
}

fn render_calendar_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let events = &app.calendar_events;
    let inner_width = area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = events
        .iter()
        .map(|ev| {
            let indicator = ev.indicator.as_deref().unwrap_or("—");
            let country = ev.country.as_deref().unwrap_or("—");
            let date = ev.date.as_deref().unwrap_or("—");
            // Trim to HH:MM if date contains a time component
            let date_display = date.get(..10).unwrap_or(date);

            let importance_marker = match ev.importance {
                Some(3) => Span::styled("● ", Style::default().fg(Color::Red)),
                Some(2) => Span::styled("● ", Style::default().fg(Color::Yellow)),
                _ => Span::styled("● ", Style::default().fg(Color::DarkGray)),
            };

            let title_line = Line::from(vec![
                importance_marker,
                Span::styled(
                    truncate(indicator, inner_width.saturating_sub(2)),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]);
            let meta_line = Line::from(Span::styled(
                format!("{country}  {date_display}"),
                Style::default().fg(Color::DarkGray),
            ));

            ListItem::new(Text::from(vec![title_line, meta_line, Line::raw("")]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Economic Calendar ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !events.is_empty() {
        state.select(Some(app.calendar_focus));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_calendar_detail(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let events = &app.calendar_events;

    if events.is_empty() {
        frame.render_widget(
            Paragraph::new("No economic events available.")
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let ev = &events[app.calendar_focus.min(events.len() - 1)];

    let indicator = ev.indicator.as_deref().unwrap_or("—");
    let country = ev.country.as_deref().unwrap_or("—");
    let date = ev.date.as_deref().unwrap_or("—");
    let currency = ev.currency.as_deref().unwrap_or("—");
    let source = ev.source.as_deref().unwrap_or("—");
    let period = ev.period.as_deref().unwrap_or("—");
    let unit = ev.unit.as_deref().unwrap_or("—");
    let scale = ev.scale.as_deref().unwrap_or("—");
    let importance_str = match ev.importance {
        Some(3) => "High",
        Some(2) => "Medium",
        Some(1) => "Low",
        _ => "—",
    };
    let importance_color = match ev.importance {
        Some(3) => Color::Red,
        Some(2) => Color::Yellow,
        _ => Color::White,
    };

    let fmt_opt = |v: Option<f64>| v.map_or("—".to_string(), |x| format!("{x:.2}"));

    let sep = "─".repeat(area.width.saturating_sub(2) as usize);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(Span::styled(
            indicator.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            format!("  {country}  ·  {date}  ·  {currency}"),
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::raw(sep),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Importance  ", Style::default().fg(Color::Yellow)),
            Span::styled(importance_str, Style::default().fg(importance_color)),
        ]),
        Line::from(vec![
            Span::styled("  Period      ", Style::default().fg(Color::Yellow)),
            Span::raw(period.to_string()),
        ]),
        Line::from(vec![
            Span::styled("  Source      ", Style::default().fg(Color::Yellow)),
            Span::raw(source.to_string()),
        ]),
        Line::from(vec![
            Span::styled("  Unit        ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{unit}  (scale: {scale})")),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Actual      ", Style::default().fg(Color::Yellow)),
            Span::raw(fmt_opt(ev.actual)),
        ]),
        Line::from(vec![
            Span::styled("  Forecast    ", Style::default().fg(Color::Yellow)),
            Span::raw(fmt_opt(ev.forecast)),
        ]),
        Line::from(vec![
            Span::styled("  Previous    ", Style::default().fg(Color::Yellow)),
            Span::raw(fmt_opt(ev.previous)),
        ]),
    ];

    if let Some(ref comment) = ev.comment {
        if !comment.is_empty() {
            let width = area.width.saturating_sub(4) as usize;
            lines.push(Line::raw(""));
            for wrapped in word_wrap(comment, width) {
                lines.push(Line::raw(format!("  {wrapped}")));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_markets_overview_tab(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.market_snapshots.is_empty() {
        frame.render_widget(
            Paragraph::new("No market data loaded.").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let sections = Layout::vertical([Constraint::Fill(1), Constraint::Length(12)]).split(area);

    let count = app.market_snapshots.len();
    let constraints: Vec<Constraint> = (0..count)
        .map(|_| Constraint::Ratio(1, count as u32))
        .collect();
    let columns = Layout::horizontal(constraints).split(sections[0]);

    for (i, (name, snapshot)) in app.market_snapshots.iter().enumerate() {
        let is_active = i == app.markets_col;
        let row_focus = app.markets_row.get(i).copied().unwrap_or(0);
        render_snapshot_block(frame, name, snapshot, columns[i], row_focus, is_active);
    }

    render_price_chart(frame, app, sections[1]);
}

fn render_snapshot_block(
    frame: &mut Frame,
    name: &str,
    snapshot: &MarketSnapshot,
    area: ratatui::layout::Rect,
    row_focus: usize,
    is_active: bool,
) {
    let header = Row::new(vec![
        Cell::from("Ticker"),
        Cell::from("Close"),
        Cell::from("Chg"),
        Cell::from("Chg %"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let mut tickers: Vec<&String> = snapshot.data.keys().collect();
    tickers.sort();

    let rows: Vec<Row> = tickers
        .iter()
        .map(|ticker| {
            let bars = &snapshot.data[*ticker];
            let Some(latest) = bars.last() else {
                return Row::new(vec![
                    Cell::from(ticker.as_str()),
                    Cell::from("—"),
                    Cell::from("—"),
                    Cell::from("—"),
                ]);
            };
            let first_open = bars.first().map(|b| b.open).unwrap_or(latest.open);
            let chg = latest.close - first_open;
            let chg_pct = if first_open != 0.0 {
                (chg / first_open) * 100.0
            } else {
                0.0
            };
            let color = if chg >= 0.0 { Color::Green } else { Color::Red };
            let sign = if chg >= 0.0 { "+" } else { "" };

            Row::new(vec![
                Cell::from(ticker.as_str()).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(format!("{:.2}", latest.close)),
                Cell::from(format!("{sign}{chg:.2}")).style(Style::default().fg(color)),
                Cell::from(format!("{sign}{chg_pct:.2}%")).style(Style::default().fg(color)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(9),
    ];

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" {name} "))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    state.select(Some(row_focus));
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_price_chart(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let col = app.markets_col;
    let focused = app.market_snapshots.values().nth(col).and_then(|snapshot| {
        let mut tickers: Vec<&String> = snapshot.data.keys().collect();
        tickers.sort();
        let row = app.markets_row.get(col).copied().unwrap_or(0);
        tickers.get(row).and_then(|ticker| {
            snapshot
                .data
                .get(*ticker)
                .map(|bars| (*ticker, bars.as_slice()))
        })
    });

    let Some((ticker, bars)) = focused else {
        frame.render_widget(Block::default().borders(Borders::ALL), area);
        return;
    };

    if bars.is_empty() {
        frame.render_widget(
            Paragraph::new("No data.").block(
                Block::default()
                    .title(format!(" {ticker} "))
                    .borders(Borders::ALL),
            ),
            area,
        );
        return;
    }

    render_line_chart(frame, ticker, bars, area);
}

fn render_line_chart(
    frame: &mut Frame,
    ticker: &str,
    bars: &[QuoteBar],
    area: ratatui::layout::Rect,
) {
    let points: Vec<(f64, f64)> = bars
        .iter()
        .enumerate()
        .map(|(i, b)| (i as f64, b.close))
        .collect();

    let min_close = bars.iter().map(|b| b.close).fold(f64::MAX, f64::min);
    let max_close = bars.iter().map(|b| b.close).fold(f64::MIN, f64::max);
    // Add a small margin so the line doesn't hug the edges
    let margin = (max_close - min_close) * 0.05;
    let y_min = min_close - margin;
    let y_max = max_close + margin;

    let x_max = (bars.len() - 1) as f64;

    // Date labels: first, middle, last bar
    let date_label = |b: &QuoteBar| {
        OffsetDateTime::from_unix_timestamp(b.timestamp)
            .map(|dt| format!("{}/{}", dt.month() as u8, dt.day()))
            .unwrap_or_default()
    };
    let x_labels = vec![
        Span::raw(date_label(&bars[0])),
        Span::raw(date_label(&bars[bars.len() / 2])),
        Span::raw(date_label(bars.last().unwrap())),
    ];

    let dataset = Dataset::default()
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&points);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(format!(" {ticker} — Close Price "))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(x_labels)
                .style(Style::default().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::raw(format!("{y_min:.2}")),
                    Span::raw(format!("{:.2}", (y_min + y_max) / 2.0)),
                    Span::raw(format!("{y_max:.2}")),
                ])
                .style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(chart, area);
}

fn render_placeholder(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let label = MAIN_TABS[app.main_tab];
    let p = Paragraph::new(format!("{label} — coming soon"))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn render_indices_tab(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let content =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).split(area);

    render_quotes_table(frame, app, content[0]);
    render_earnings_table(frame, app, content[1]);
}

fn render_portfolio_tab(
    frame: &mut Frame,
    app: &App,
    area: ratatui::layout::Rect,
    portfolio_idx: usize,
) {
    let portfolio = &app.portfolios[portfolio_idx];

    let header = Row::new(vec![
        Cell::from("Symbol"),
        Cell::from("Qty"),
        Cell::from("Cost Basis"),
        Cell::from("Curr Price"),
        Cell::from("Mkt Value"),
        Cell::from("Gain/Loss"),
        Cell::from("G/L %"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = portfolio
        .positions
        .iter()
        .map(|pos| {
            let quote = app.portfolio_quotes.get(&pos.ticker);
            let (price_str, value_str, gl_str, gl_pct_str, gl_color) = if let Some(q) = quote {
                let mkt_value = q.current_price * pos.quantity;
                let cost_total = pos.cost_basis * pos.quantity;
                let gl = mkt_value - cost_total;
                let gl_pct = (gl / cost_total) * 100.0;
                let color = if gl >= 0.0 { Color::Green } else { Color::Red };
                let sign = if gl >= 0.0 { "+" } else { "" };
                (
                    format!("${:.2}", q.current_price),
                    format!("${:.2}", mkt_value),
                    format!("{}{:.2}", sign, gl),
                    format!("{}{:.2}%", sign, gl_pct),
                    color,
                )
            } else {
                (
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    Color::White,
                )
            };

            Row::new(vec![
                Cell::from(pos.ticker.clone()).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(format!("{:.0}", pos.quantity)),
                Cell::from(format!("${:.2}", pos.cost_basis)),
                Cell::from(price_str),
                Cell::from(value_str),
                Cell::from(gl_str).style(Style::default().fg(gl_color)),
                Cell::from(gl_pct_str).style(Style::default().fg(gl_color)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" {} Positions ", portfolio.portfolio_name))
                .borders(Borders::ALL),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    state.select(app.portfolio_focus[portfolio_idx]);
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_title(frame: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " CEREBRO Dashboard ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let key = |label: &'static str| {
        Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    };

    // While the research palette is open, show only the input-mode hints.
    let spans: Vec<Span> = if app.is_research_inputting() {
        vec![
            key("Enter"),
            Span::raw("search  "),
            key("Esc"),
            Span::raw("cancel"),
        ]
    } else {
        let mut s = vec![
            key("q"),
            Span::raw("quit  "),
            key("← →"),
            Span::raw("section"),
        ];
        match app.main_tab {
            0 => s.extend([
                Span::raw("  "),
                key("[ ]"),
                Span::raw("column  "),
                key("↑ ↓"),
                Span::raw("navigate"),
            ]),
            1 => {
                s.extend([Span::raw("  "), key("[ ]"), Span::raw("sub-tab")]);
                if app.active_tab > 0 {
                    s.extend([
                        Span::raw("  "),
                        key("↑ ↓"),
                        Span::raw("select  "),
                        key("Enter"),
                        Span::raw("open in browser"),
                    ]);
                }
            }
            2 => s.extend([
                Span::raw("  "),
                key("[ ]"),
                Span::raw("sub-tab  "),
                key("↑ ↓"),
                Span::raw("navigate  "),
                key("o"),
                Span::raw("open  "),
                key("r"),
                Span::raw("refresh"),
            ]),
            3 => {
                s.extend([Span::raw("  "), key(":"), Span::raw("search")]);
                if app.research_quote.is_some() {
                    s.extend([Span::raw("  "), key("[ ]"), Span::raw("sub-tab")]);
                    match app.research_sub_tab {
                        1 => s.extend([
                            Span::raw("  "),
                            key("↑ ↓"),
                            Span::raw("navigate  "),
                            key("o"),
                            Span::raw("open"),
                        ]),
                        2 => s.extend([Span::raw("  "), key("↑ ↓"), Span::raw("navigate")]),
                        _ => {}
                    }
                }
            }
            4 => s.extend([Span::raw("  "), key("↑ ↓"), Span::raw("navigate")]),
            _ => {}
        }
        s
    };

    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_quotes_table(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from("Symbol"),
        Cell::from("Price"),
        Cell::from("Change"),
        Cell::from("% Change"),
        Cell::from("High"),
        Cell::from("Low"),
        Cell::from("Open"),
        Cell::from("Prev Close"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .quotes
        .iter()
        .map(|q| {
            let change_color = if q.change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let change_sign = if q.change >= 0.0 { "+" } else { "" };

            Row::new(vec![
                Cell::from(q.symbol.clone()).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(format!("${:.2}", q.current_price)),
                Cell::from(format!("{}{:.2}", change_sign, q.change))
                    .style(Style::default().fg(change_color)),
                Cell::from(format!("{}{:.2}%", change_sign, q.percent_change))
                    .style(Style::default().fg(change_color)),
                Cell::from(format!("${:.2}", q.high)),
                Cell::from(format!("${:.2}", q.low)),
                Cell::from(format!("${:.2}", q.open)),
                Cell::from(format!("${:.2}", q.previous_close)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(" Stock Quotes ")
                .borders(Borders::ALL),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(table, area);
}

fn render_earnings_table(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from("Symbol"),
        Cell::from("Date"),
        Cell::from("Hour"),
        Cell::from("Q"),
        Cell::from("Year"),
        Cell::from("EPS Est"),
        Cell::from("EPS Act"),
        Cell::from("Rev Est"),
        Cell::from("Rev Act"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .earnings
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(r.symbol.as_deref().unwrap_or("—").to_string()),
                Cell::from(r.date.as_deref().unwrap_or("—").to_string()),
                Cell::from(r.hour.as_deref().unwrap_or("—").to_string()),
                Cell::from(r.quarter.map_or("—".to_string(), |v| v.to_string())),
                Cell::from(r.year.map_or("—".to_string(), |v| v.to_string())),
                Cell::from(
                    r.eps_estimate
                        .map_or("—".to_string(), |v| format!("{v:.2}")),
                ),
                Cell::from(r.eps_actual.map_or("—".to_string(), |v| format!("{v:.2}"))),
                Cell::from(
                    r.revenue_estimate
                        .map_or("—".to_string(), |v| format!("{:.0}", v / 1_000_000.0))
                        + "M",
                ),
                Cell::from(
                    r.revenue_actual
                        .map_or("—".to_string(), |v| format!("{:.0}", v / 1_000_000.0))
                        + "M",
                ),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Length(6),
        Constraint::Length(3),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(" Earnings Calendar ")
                .borders(Borders::ALL),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(table, area);
}
