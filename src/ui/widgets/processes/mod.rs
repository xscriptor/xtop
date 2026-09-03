//! Processes widget: sortable live process table with search.

use crate::state::AppState;
use crate::ui::share::to_color;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use xtop_plugin_api::model::ProcessInfo;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let dim_bg = to_color(&state.current_theme.palette[8]);
    let accent = to_color(&state.current_theme.palette[6]);

    let mut title = format!("Processes (sort: {})", state.process_sort.label());
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

    let iter: Box<dyn Iterator<Item = &ProcessInfo>> = if state.search_query.is_empty() {
        Box::new(snap.processes.iter())
    } else {
        let q = state.search_query.to_lowercase();
        Box::new(
            snap.processes
                .iter()
                .filter(move |p| p.name.to_lowercase().contains(&q)),
        )
    };

    let mut items: Vec<&ProcessInfo> = iter.collect();

    // Sort
    match state.process_sort {
        crate::state::ProcessSortBy::Cpu => {
            items.sort_by(|a, b| {
                b.cpu_usage
                    .partial_cmp(&a.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        crate::state::ProcessSortBy::Memory => {
            items.sort_by_key(|b| std::cmp::Reverse(b.memory));
        }
        crate::state::ProcessSortBy::Pid => {
            items.sort_by_key(|a| a.pid);
        }
        crate::state::ProcessSortBy::Name => {
            items.sort_by_key(|a| a.name.to_lowercase());
        }
    }

    let rows: Vec<Row> = items
        .into_iter()
        .enumerate()
        .map(|(row_idx, p)| {
            let is_selected = state.process_selected == Some(row_idx);
            let style = if is_selected {
                Style::default()
                    .fg(bg)
                    .bg(accent)
                    .add_modifier(Modifier::BOLD)
            } else if row_idx % 2 == 0 {
                Style::default().fg(fg)
            } else {
                Style::default().fg(fg).bg(dim_bg)
            };
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}%", p.cpu_usage)),
                Cell::from(crate::ui::share::format_bytes(p.memory)),
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
                .style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .row_highlight_style(
            Style::default()
                .fg(bg)
                .bg(accent)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(table, inner);
}
