use crate::color::to_color;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use xtop_core::application::state::{AppState, PalettePage};

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let accent = to_color(&state.current_theme.palette[6]);

    let popup_width = (area.width as f64 * 0.6).min(60.0) as u16;
    let popup_height = (area.height as f64 * 0.6).min(30.0) as u16;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    let title = state.palette.title();
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let hint = if state.palette.page == PalettePage::Main {
        " Filter: type to search · navigate with ↑↓ · select with Enter"
    } else {
        " Filter themes · Enter to select · Esc/Bksp to go back"
    };
    let input_label = match state.palette.page {
        PalettePage::Main => "Action",
        PalettePage::Themes => "Theme",
        PalettePage::Layouts => "Layout",
    };

    let search_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 3,
    };
    let input_text = if state.palette.query.is_empty() {
        format!(" {}_", hint)
    } else {
        format!(" {}: {}", input_label, state.palette.query)
    };
    let input = Paragraph::new(input_text.as_str())
        .style(Style::default().fg(accent).bg(bg))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(input, search_area);

    let list_area = Rect {
        x: inner.x,
        y: inner.y + 3,
        width: inner.width,
        height: inner.height.saturating_sub(3).max(1),
    };

    let items: Vec<ListItem> = state
        .palette
        .filtered
        .iter()
        .map(|&entry_idx| {
            let entry = &state.palette.entries[entry_idx];
            ListItem::new(entry.label.as_str()).style(Style::default().fg(fg))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.palette.selected));

    let list = List::new(items)
        .highlight_style(Style::default().fg(bg).bg(fg).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");
    f.render_stateful_widget(list, list_area, &mut list_state);
}
