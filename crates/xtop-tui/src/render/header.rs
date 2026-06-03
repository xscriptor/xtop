use crate::format::format_uptime;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use xtop_core::application::state::{AppState, FullScreenWidget, InputMode};

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());

    let load = state.snapshot().load_avg;
    let uptime = state.snapshot().uptime;

    let mode_str = state.layout_mode.label();

    let mut extras = String::new();
    if state.full_screen_widget != FullScreenWidget::None {
        extras.push_str(&format!(" [Full: {}]", state.full_screen_widget.label()));
    }
    if state.input_mode == InputMode::Searching {
        extras.push_str(" [/] Search");
    }

    let text = format!(
        "xtop | Theme: {} | Layout: {}{} | Uptime: {} | Load: {:.2} {:.2} {:.2} | [q] Quit [?] Help [t] Theme [l] Layout [f] Full [/] Search",
        state.current_theme.name,
        mode_str,
        extras,
        format_uptime(uptime),
        load.one,
        load.five,
        load.fifteen,
    );

    let p = Paragraph::new(text)
        .style(Style::default().fg(fg).bg(bg))
        .block(Block::default().borders(Borders::ALL).title("System Info"));
    f.render_widget(p, area);
}
