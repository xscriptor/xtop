use crate::color::to_color;
use crate::format::format_uptime;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use xtop_core::application::state::{AppState, FullScreenWidget, InputMode};

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let snap = state.snapshot();
    let load = snap.load_avg;
    let uptime = snap.uptime;

    let mode_str = state.current_layout_name();

    let mut extras = String::new();
    if state.full_screen_widget != FullScreenWidget::None {
        extras.push_str(&format!(" [Full: {}]", state.full_screen_widget.label()));
    }
    if state.input_mode == InputMode::Searching {
        extras.push_str(" [/] Search");
    }

    let host = &state.sys_info.hostname;

    let wide = area.width >= 80;
    let text: Vec<Line> = if wide {
        vec![Line::from(format!(
            "{} | {} | {} | Uptime: {} | Load: {:.2} {:.2} {:.2}{}",
            if host.is_empty() { "xtop".to_string() } else { host.clone() },
            state.current_theme.name,
            mode_str,
            format_uptime(uptime),
            load.one,
            load.five,
            load.fifteen,
            extras,
        ))]
    } else {
        let host_part = if host.is_empty() {
            mode_str.to_string()
        } else {
            format!("{} | {}", host, mode_str)
        };
        vec![
            Line::from(format!(
                "{} | Uptime: {}",
                host_part,
                format_uptime(uptime),
            )),
            Line::from(format!(
                "Load: {:.2} {:.2} {:.2}{}",
                load.one, load.five, load.fifteen, extras,
            )),
        ]
    };

    let p = Paragraph::new(text)
        .style(Style::default().fg(fg).bg(bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::PLAIN)
                .title("System Info"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
