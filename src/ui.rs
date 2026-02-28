use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs, Wrap},
};

use crate::app::{App, MAIN_TABS, NEWS_TABS, TAB_TITLES};

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
    let panes = Layout::horizontal([
        Constraint::Percentage(35),
        Constraint::Percentage(65),
    ])
    .split(area);

    render_news_list(frame, app, panes[0]);
    render_news_detail(frame, app, panes[1]);
}

fn render_news_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let articles = &app.news_articles[app.news_tab];
    let category_label = NEWS_TABS[app.news_tab];

    let items: Vec<ListItem> = articles
        .iter()
        .map(|a| ListItem::new(a.headline.clone()))
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
        .highlight_symbol("> ");

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
            Paragraph::new("No articles available.")
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let article = &articles[app.news_focus.min(articles.len() - 1)];

    let rel_time = relative_time(article.datetime);
    let meta = format!("  {} • {} • {}", article.source, rel_time, article.category);
    let sep = "─".repeat(area.width.saturating_sub(2) as usize);
    let url_display = strip_scheme(&article.url);

    // Manually word-wrap the summary so every line gets the 2-space left pad.
    let summary_width = area.width.saturating_sub(4) as usize; // borders (2) + padding (2)
    let wrapped_summary = word_wrap(&article.summary, summary_width);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(Span::styled(
            article.headline.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            meta,
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::raw(sep),
    ];

    for summary_line in wrapped_summary {
        lines.push(Line::raw(format!("  {summary_line}")));
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

    let detail = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, area);
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

    let mut spans = vec![
        key("q"),
        Span::raw("quit  "),
        key("← →"),
        Span::raw("section"),
    ];

    match app.main_tab {
        0 => {
            spans.extend([Span::raw("  "), key("[ ]"), Span::raw("sub-tab")]);
            if app.active_tab > 0 {
                spans.extend([
                    Span::raw("  "),
                    key("↑ ↓"),
                    Span::raw("select  "),
                    key("Enter"),
                    Span::raw("open in browser"),
                ]);
            }
        }
        1 => {
            spans.extend([
                Span::raw("  "),
                key("[ ]"),
                Span::raw("sub-tab  "),
                key("↑ ↓"),
                Span::raw("navigate  "),
                key("o"),
                Span::raw("open  "),
                key("r"),
                Span::raw("refresh"),
            ]);
        }
        _ => {}
    }

    let status = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, area);
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
