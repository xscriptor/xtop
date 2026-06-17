use crate::color::to_color;
use crate::format::format_bytes;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let block = Block::default()
        .title("Storage")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    let disks = &snap.disks;
    if disks.is_empty() {
        return;
    }

    let per_disk = inner.height.min(3);
    let constraints = vec![Constraint::Length(per_disk); disks.len()];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, disk) in disks.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        let is_alert = disk.percent > state.alerts.disk_high;
        let color_idx = if is_alert { 1 } else { 4 };
        let label = format!(
            "{}  Tot: {}  Use: {}  Free: {}",
            disk.mount_point,
            format_bytes(disk.total_space),
            format_bytes(disk.used_space),
            format_bytes(disk.available_space),
        );
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(to_color(&state.current_theme.palette[color_idx]))
                    .bg(bg),
            )
            .percent(disk.percent as u16)
            .label(label);
        f.render_widget(gauge, chunks[i]);
    }
}
