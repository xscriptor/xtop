use crate::color::to_color;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let mut title = "Processes".to_string();
    if !state.search_query.is_empty() {
        title = format!("Processes (filter: {})", state.search_query);
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::PLAIN)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();

    let iter: Box<dyn Iterator<Item = &xtop_core::domain::metrics::ProcessInfo>> =
        if state.search_query.is_empty() {
            Box::new(snap.processes.iter())
        } else {
            let q = state.search_query.to_lowercase();
            Box::new(
                snap.processes
                    .iter()
                    .filter(move |p| p.name.to_lowercase().contains(&q)),
            )
        };

    let dim_bg = to_color(&state.current_theme.palette[8]);

    let rows: Vec<Row> = iter
        .enumerate()
        .map(|(row_idx, p)| {
            let style = if row_idx % 2 == 0 {
                Style::default().fg(fg)
            } else {
                Style::default().fg(fg).bg(dim_bg)
            };
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}%", p.cpu_usage)),
                Cell::from(crate::format::format_bytes(p.memory)),
                Cell::from(p.user_id.clone().unwrap_or_else(|| "?".to_string())),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(40),
        Constraint::Length(12),
        Constraint::Length(17),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["PID", "Name", "CPU%", "Mem", "User"])
                .style(
                    Style::default()
                        .fg(to_color(&state.current_theme.palette[6]))
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(table, inner);
}
