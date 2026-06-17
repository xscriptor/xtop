use crate::color::to_color;
use crate::format::format_bytes;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let block = Block::default()
        .title("Disk I/O")
        .borders(Borders::ALL)
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

    let mut lines = Vec::new();
    for d in &snap.disk_io {
        let read_speed = format_bytes(d.read_speed as u64);
        let write_speed = format_bytes(d.write_speed as u64);
        let total_read = format_bytes(d.read_bytes);
        let total_write = format_bytes(d.write_bytes);
        lines.push(Line::from(Span::raw(format!(
            " {}  R: {}/s  W: {}/s",
            d.name, read_speed, write_speed,
        ))));
        lines.push(Line::from(Span::raw(format!(
            "     Tot R: {}  Tot W: {}",
            total_read, total_write,
        ))));
    }

    let p = Paragraph::new(lines)
        .style(Style::default().fg(fg))
        .wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}
