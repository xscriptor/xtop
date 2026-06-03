use crate::format::format_bytes;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());

    let block = Block::default()
        .title("Disk I/O")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    if snap.disk_io.is_empty() {
        return;
    }

    let per_disk = inner.height.min(3);
    let constraints = vec![Constraint::Length(per_disk); snap.disk_io.len()];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, d) in snap.disk_io.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        let total_read = format_bytes(d.read_bytes);
        let total_write = format_bytes(d.write_bytes);
        let read_speed = format_bytes(d.read_speed as u64);
        let write_speed = format_bytes(d.write_speed as u64);
        let label = format!(
            "{}  R: {}/s  W: {}/s  Tot R: {}  Tot W: {}",
            d.name, read_speed, write_speed, total_read, total_write,
        );
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(rgb(&state.current_theme.palette[4]))
                    .bg(bg),
            )
            .percent(50)
            .label(label);
        f.render_widget(gauge, chunks[i]);
    }
}
