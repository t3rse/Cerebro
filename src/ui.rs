use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(frame.area());

    render_title(frame, outer[0]);

    if app.loading {
        let loading = Paragraph::new("Loading data...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, outer[1]);
    } else {
        let content = Layout::horizontal([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ])
        .split(outer[1]);

        render_quote_panel(frame, app, content[0]);
        render_earnings_table(frame, app, content[1]);
    }

    render_status_bar(frame, outer[2]);
}

fn render_title(frame: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " CEREBRO Dashboard ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_status_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" q ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("quit"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, area);
}

fn render_quote_panel(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .title(" Stock Quote ")
        .borders(Borders::ALL);

    let content = match &app.quote {
        Some(q) => {
            let change_color = if q.change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let change_sign = if q.change >= 0.0 { "+" } else { "" };

            vec![
                Line::from(vec![
                    Span::styled(
                        format!(" {} ", q.symbol),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Price:      "),
                    Span::styled(
                        format!("${:.2}", q.current_price),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Change:     "),
                    Span::styled(
                        format!("{}{:.2} ({}{:.2}%)", change_sign, q.change, change_sign, q.percent_change),
                        Style::default().fg(change_color),
                    ),
                ]),
                Line::from(format!("  High:       ${:.2}", q.high)),
                Line::from(format!("  Low:        ${:.2}", q.low)),
                Line::from(format!("  Open:       ${:.2}", q.open)),
                Line::from(format!("  Prev Close: ${:.2}", q.previous_close)),
            ]
        }
        None => vec![Line::from("  No quote data available")],
    };

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
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
                Cell::from(r.eps_estimate.map_or("—".to_string(), |v| format!("{v:.2}"))),
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
