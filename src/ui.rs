use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs},
};

use crate::app::{App, MAIN_TABS, TAB_TITLES};

pub fn render(frame: &mut Frame, app: &App) {
    let on_portfolios = app.main_tab == 0;

    // Build a layout that includes the sub-tabs row only under Portfolios.
    let constraints: Vec<Constraint> = if on_portfolios {
        vec![
            Constraint::Length(3), // title
            Constraint::Length(3), // main nav
            Constraint::Length(3), // portfolio sub-tabs
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

    if on_portfolios {
        render_portfolio_sub_tabs(frame, app, outer[2]);
        let content_area = outer[3];
        let status_area = outer[4];
        if app.loading {
            frame.render_widget(
                Paragraph::new("Loading data...").block(Block::default().borders(Borders::ALL)),
                content_area,
            );
        } else {
            match app.active_tab {
                0 => render_indices_tab(frame, app, content_area),
                1 => render_portfolio_tab(frame, app, content_area, 0),
                2 => render_portfolio_tab(frame, app, content_area, 1),
                _ => {}
            }
        }
        render_status_bar(frame, app, status_area);
    } else {
        let content_area = outer[2];
        let status_area = outer[3];
        render_placeholder(frame, app, content_area);
        render_status_bar(frame, app, status_area);
    }
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

    if app.main_tab == 0 {
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
