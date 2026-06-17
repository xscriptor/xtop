use crate::color::to_color;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let popup_width = (area.width as f64 * 0.6) as u16;
    let popup_height = (area.height as f64 * 0.6) as u16;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    let block = Block::default()
        .title("Command Palette")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let search_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 3,
    };
    let input_text = format!(
        " {}_",
        if state.palette.query.is_empty() {
            String::new()
        } else {
            state.palette.query.clone()
        }
    );
    let input = Paragraph::new(input_text.as_str())
        .style(Style::default().fg(fg).bg(bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search commands"),
        );
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
        .enumerate()
        .map(|(idx, &entry_idx)| {
            let entry = &state.palette.entries[entry_idx];
            let is_selected = idx == state.palette.selected;
            let style = if is_selected {
                Style::default()
                    .fg(bg)
                    .bg(fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg)
            };
            ListItem::new(entry.label.as_str()).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, list_area);
}
