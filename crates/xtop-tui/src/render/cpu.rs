use crate::color::to_color;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let snap = state.snapshot();

    let title = if snap.cpu_temp > 0.0 {
        format!("CPU (Max: {:.1}°C)", snap.cpu_temp)
    } else {
        "CPU".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if snap.cpus.is_empty() {
        return;
    }

    let count = snap.cpus.len();
    let cols = if inner.width > 40 { 2 } else { 1 };
    let col_constraints = if cols == 2 {
        vec![Constraint::Percentage(50); 2]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let col_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(col_constraints)
        .split(inner);

    let per_col = count.div_ceil(cols);

    for (col_idx, col_area) in col_areas.iter().enumerate() {
        let start = col_idx * per_col;
        let end = (start + per_col).min(count);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); end - start])
            .split(*col_area);

        for (i, row_area) in rows.iter().enumerate() {
            let cpu_idx = start + i;
            if cpu_idx >= count {
                break;
            }
            let cpu = &snap.cpus[cpu_idx];
            let usage = cpu.usage;
            let is_alert = usage > state.alerts.cpu_high;
            let label = format!("CPU{:<2} {:>3.0}%", cpu.cpu_id, usage);
            let color_idx = if is_alert { 1 } else { 1 + (cpu.cpu_id % 6) };

            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(to_color(&state.current_theme.palette[color_idx]))
                        .bg(bg),
                )
                .percent(usage as u16)
                .label(label);
            f.render_widget(gauge, *row_area);
        }
    }
}
