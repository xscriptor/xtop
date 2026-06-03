use crate::format::format_bytes;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());

    let block = Block::default()
        .title("Network")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    let total_rx: u64 = snap.networks.iter().map(|n| n.received).sum();
    let total_tx: u64 = snap.networks.iter().map(|n| n.transmitted).sum();

    let text = vec![
        Line::from(vec![
            Span::styled("Total RX: ", Style::default().fg(fg)),
            Span::styled(
                format_bytes(total_rx),
                Style::default().fg(rgb(&state.current_theme.palette[4])),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total TX: ", Style::default().fg(fg)),
            Span::styled(
                format_bytes(total_tx),
                Style::default().fg(rgb(&state.current_theme.palette[5])),
            ),
        ]),
    ];

    let p = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}
