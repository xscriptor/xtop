use crate::color::to_color;
use crate::format::format_uptime;
use ratatui::prelude::*;
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

    let wide = area.width >= 80;
    let text: Vec<Line> = if wide {
        vec![Line::from(format!(
            "xtop | {} | {} | Uptime: {} | Load: {:.2} {:.2} {:.2} | [q] [?] [t] [T] [l] [f] [/]",
            state.current_theme.name,
            mode_str,
            format_uptime(uptime),
            load.one,
            load.five,
            load.fifteen,
        ))]
    } else {
        vec![
            Line::from(format!(
                "{} | Uptime: {}",
                mode_str,
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
        .block(Block::default().borders(Borders::ALL).title("System Info"))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
