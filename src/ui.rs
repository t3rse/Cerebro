use crate::app::{App, MAIN_TABS, NEWS_TABS, TAB_TITLES};
use ratatui::text::Text;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs,
        Wrap,
    },
};

pub fn render(frame: &mut Frame, app: &App) {
    // Portfolios (0) and News (1) both have a sub-tabs row.
    let has_sub_tabs = app.main_tab <= 1;

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
            0 => render_portfolio_sub_tabs(frame, app, outer[2]),
            1 => render_news_sub_tabs(frame, app, outer[2]),
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
            0 => match app.active_tab {
                0 => render_indices_tab(frame, app, content_area),
                1 => render_portfolio_tab(frame, app, content_area, 0),
                2 => render_portfolio_tab(frame, app, content_area, 1),
                _ => {}
            },
            1 => render_news_tab(frame, app, content_area),
            2 => render_research_tab(frame, app, content_area),
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
        let rows =
            Layout::vertical([Constraint::Length(6), Constraint::Fill(1)]).split(content_area);
        render_research_quote(frame, app, rows[0]);
        let panes = Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(rows[1]);
        render_research_news_list(frame, app, panes[0]);
        render_research_news_detail(frame, app, panes[1]);
    } else {
        render_research_empty(frame, content_area);
    }
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

fn render_research_news_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let articles = &app.research_news;
    let inner_width = area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = articles
        .iter()
        .map(|a| {
            let headline = truncate(&a.headline, inner_width);
            let meta = truncate(
                &format!("{} · {}", a.source, relative_time(a.datetime)),
                inner_width,
            );
            ListItem::new(Text::from(vec![
                Line::from(Span::styled(
                    headline,
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(meta, Style::default().fg(Color::DarkGray))),
                Line::raw(""),
            ]))
        })
        .collect();

    let symbol = app.research_symbol.as_deref().unwrap_or("Symbol");
    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" {symbol} News "))
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
        state.select(Some(app.research_news_focus));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_research_news_detail(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let articles = &app.research_news;

    if articles.is_empty() {
        frame.render_widget(
            Paragraph::new("No recent news found.").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let a = &articles[app.research_news_focus.min(articles.len() - 1)];
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
            0 => {
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
            1 => s.extend([
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
            2 => {
                s.extend([Span::raw("  "), key(":"), Span::raw("search")]);
                if app.research_quote.is_some() {
                    s.extend([
                        Span::raw("  "),
                        key("↑ ↓"),
                        Span::raw("navigate  "),
                        key("o"),
                        Span::raw("open"),
                    ]);
                }
            }
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
