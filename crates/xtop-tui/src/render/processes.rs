use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());

    let mut title = "Processes".to_string();
    if !state.search_query.is_empty() {
        title = format!("Processes (filter: {})", state.search_query);
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    let separator = Span::styled(
        " | ",
        Style::default().fg(rgb(&state.current_theme.palette[8])),
    );

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

    let rows: Vec<Row> = iter
        .map(|p| {
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::raw(p.pid.to_string()),
                    separator.clone(),
                ])),
                Cell::from(Line::from(vec![
                    Span::raw(p.name.clone()),
                    separator.clone(),
                ])),
                Cell::from(Line::from(vec![
                    Span::raw(format!("{:.1}%", p.cpu_usage)),
                    separator.clone(),
                ])),
                Cell::from(Line::from(vec![
                    Span::raw(crate::format::format_bytes(p.memory)),
                    separator.clone(),
                ])),
                Cell::from(p.user_id.clone().unwrap_or_else(|| "?".to_string())),
            ])
            .style(Style::default().fg(fg))
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
            Row::new(vec!["PID |", "Name |", "CPU% |", "Mem |", "User"])
                .style(
                    Style::default()
                        .fg(rgb(&state.current_theme.palette[6]))
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(table, inner);
}
