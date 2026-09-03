//! Disk I/O widget: read/write throughput.

use crate::state::AppState;
use crate::ui::share::format_bytes;
use crate::ui::share::to_color;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let block = Block::default()
        .title("Disk I/O")
        .borders(Borders::ALL)
        .border_set(border::PLAIN)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    if snap.disk_io.is_empty() {
        let msg = Paragraph::new("No disk I/O data")
            .style(Style::default().fg(fg))
            .wrap(Wrap { trim: true });
        f.render_widget(msg, inner);
        return;
    }

    // Find max speed for proportional gauge
    let max_speed = snap
        .disk_io
        .iter()
        .map(|d| d.read_speed.max(d.write_speed))
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let per_disk = 3.min(inner.height / snap.disk_io.len().max(1) as u16);
    let per_disk = per_disk.max(2);
    let constraints = vec![Constraint::Length(per_disk); snap.disk_io.len()];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, d) in snap.disk_io.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        let read_speed = format_bytes(d.read_speed as u64);
        let write_speed = format_bytes(d.write_speed as u64);

        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(to_color(&state.current_theme.palette[4]))
                    .bg(bg),
            )
            .percent((d.read_speed / max_speed * 100.0) as u16)
            .label(format!(" {}  R: {}/s", d.name, read_speed));
        f.render_widget(gauge, chunks[i]);

        // Draw write speed as a second line if there's room
        if per_disk >= 3 {
            let sub = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .split(chunks[i]);
            let write_gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(to_color(&state.current_theme.palette[5]))
                        .bg(bg),
                )
                .percent((d.write_speed / max_speed * 100.0) as u16)
                .label(format!(
                    "     W: {}/s  Tot R: {}  Tot W: {}",
                    write_speed,
                    format_bytes(d.read_bytes),
                    format_bytes(d.write_bytes)
                ));
            f.render_widget(write_gauge, sub[1]);
        }
    }
}
