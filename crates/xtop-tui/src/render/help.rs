use crate::color::to_color;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Keybindings",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  ─────────────────────────────────────────────"),
        Line::from("  q            Quit application"),
        Line::from("  ?            Toggle this help screen"),
        Line::from(""),
        Line::from("  t            Next color theme"),
        Line::from("  T            Previous color theme"),
        Line::from("  l            Next layout mode"),
        Line::from("  f            Toggle fullscreen widget"),
        Line::from("  F            Cycle fullscreen widget"),
        Line::from(""),
        Line::from("  /            Search/filter processes"),
        Line::from("  Esc          Cancel search / close help"),
        Line::from(""),
        Line::from("  Layout modes:"),
        Line::from(format!("    Current: {}", state.current_layout_name())),
        Line::from("    Dashboard | Vertical | Horizontal | CPU Focus"),
        Line::from("    Memory Focus | Network Focus | Process Focus"),
        Line::from("    + custom layouts from ~/.config/xtop/layouts/"),
        Line::from(""),
        Line::from("  ─────────────────────────────────────────────"),
        Line::from(""),
        Line::from("  https://github.com/xscriptor/xtop"),
        Line::from(""),
    ];

    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let p = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(fg).bg(bg))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}
