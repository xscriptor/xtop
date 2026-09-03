//! GPU widget: driver-reported GPU usage.

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
        .title("GPU")
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
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
                    .fg(to_color(&state.current_theme.palette[5]))
                    .bg(bg),
            )
            .percent(gpu.usage as u16)
            .label(label);
        f.render_widget(gauge, chunks[i]);
    }
}
