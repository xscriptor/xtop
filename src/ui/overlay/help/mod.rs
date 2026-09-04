//! Help widget: keybinding reference overlay.
//!
//! Built from the *live* `Keybindings` (config-driven), so remapped keys are
//! always reflected here.

use crate::state::AppState;
use crate::ui::share::to_color;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let accent = to_color(&state.current_theme.palette[6]);
    let kb = &state.keybindings;

    let mut text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Keybindings",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  ─────────────────────────────────────────────"),
    ];
    push_key(&mut text, "Quit", &kb.quit, accent);
    push_key(&mut text, "Help", &kb.help, accent);
    text.push(Line::from(""));
    push_key(&mut text, "Next theme", &kb.next_theme, accent);
    push_key(&mut text, "Previous theme", &kb.prev_theme, accent);
    push_key(&mut text, "Next layout", &kb.next_layout, accent);
    push_key(
        &mut text,
        "Toggle fullscreen",
        &kb.toggle_fullscreen,
        accent,
    );
    push_key(&mut text, "Cycle fullscreen", &kb.cycle_fullscreen, accent);
    text.push(Line::from(""));
    push_key(&mut text, "Search processes", &kb.search, accent);
    push_key(&mut text, "Command palette", &kb.command_palette, accent);
    push_key(&mut text, "Cancel", &kb.cancel, accent);
    text.push(Line::from(""));
    push_key(&mut text, "Kill process", &kb.kill_process, accent);
    push_key(&mut text, "Select up", &kb.process_up, accent);
    push_key(&mut text, "Select down", &kb.process_down, accent);
    push_key(&mut text, "Cycle sort", &kb.cycle_sort, accent);
    text.extend([
        Line::from(""),
        Line::from("  ─────────────────────────────────────────────"),
        Line::from(""),
        Line::from("  Layout modes:"),
        Line::from(format!("    Current: {}", state.current_layout_name())),
        Line::from("    Dashboard | Vertical | Horizontal | CPU Focus"),
        Line::from("    Memory Focus | Network Focus | Process Focus"),
        Line::from("    + custom layouts from ~/.config/xtop/layouts/"),
        Line::from(""),
        Line::from("  https://github.com/xtop-cli/xtop"),
    ]);

    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .style(Style::default().fg(fg).bg(bg));
    let p = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(fg).bg(bg))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn push_key(text: &mut Vec<Line<'static>>, action: &str, keys: &[String], accent: Color) {
    let rendered = if keys.is_empty() {
        "(unbound)".to_string()
    } else {
        keys.join(", ")
    };
    text.push(Line::from(vec![
        Span::styled(format!("  {rendered:<12}"), Style::default().fg(accent)),
        Span::raw(action.to_string()),
    ]));
}
