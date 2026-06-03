use crate::format::format_bytes;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());

    let block = Block::default()
        .title("GPU")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    if snap.gpus.is_empty() {
        let msg = Paragraph::new("No GPU data available")
            .style(Style::default().fg(fg))
            .wrap(Wrap { trim: true });
        f.render_widget(msg, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); snap.gpus.len()])
        .split(inner);

    for (i, gpu) in snap.gpus.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        let label = format!(
            "{}  {:>3.0}%  Mem: {} / {}  Temp: {:.1}°C",
            gpu.name,
            gpu.usage,
            format_bytes(gpu.memory_used),
            format_bytes(gpu.memory_total),
            gpu.temperature,
        );
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(rgb(&state.current_theme.palette[5]))
                    .bg(bg),
            )
            .percent(gpu.usage as u16)
            .label(label);
        f.render_widget(gauge, chunks[i]);
    }
}
