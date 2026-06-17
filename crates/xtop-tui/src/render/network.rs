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
        .title("Network")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    let total_rx: u64 = snap.networks.iter().map(|n| n.received).sum();
    let total_tx: u64 = snap.networks.iter().map(|n| n.transmitted).sum();
    let total_rx_speed: f64 = snap.networks.iter().map(|n| n.rx_speed).sum();
    let total_tx_speed: f64 = snap.networks.iter().map(|n| n.tx_speed).sum();

    let mut text = vec![
        Line::from(vec![
            Span::styled("RX: ", Style::default().fg(fg)),
            Span::styled(
                format_bytes(total_rx),
                Style::default().fg(to_color(&state.current_theme.palette[4])),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}/s", format_bytes(total_rx_speed as u64)),
                Style::default().fg(to_color(&state.current_theme.palette[4])),
            ),
        ]),
        Line::from(vec![
            Span::styled("TX: ", Style::default().fg(fg)),
            Span::styled(
                format_bytes(total_tx),
                Style::default().fg(to_color(&state.current_theme.palette[5])),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}/s", format_bytes(total_tx_speed as u64)),
                Style::default().fg(to_color(&state.current_theme.palette[5])),
            ),
        ]),
    ];

    if inner.height > 4 {
        for iface in &snap.networks {
            if text.len() as u16 >= inner.height.saturating_sub(1) {
                break;
            }
            text.push(Line::from(Span::raw(format!(
                " {}  RX: {}  TX: {}",
                iface.name,
                format_bytes(iface.received),
                format_bytes(iface.transmitted),
            ))));
        }
    }

    let p = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}
