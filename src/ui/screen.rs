//! Screen renderer: dispatches to layouts, fullscreen, overlays and the
//! minimal view. Data widgets are drawn through the widget packs resolved by
//! the engine; overlays (help/palette) are kernel-owned.

use crate::state::{AppState, FullScreenWidget, InputMode};
use crate::ui::layout::{render_layout, render_named, PluginWidgetFn};
use crate::ui::overlay::{help, palette};
use crate::ui::share::to_color;
use ratatui::prelude::*;
use ratatui::Frame;
use std::collections::HashMap;
use xtop_layout::{detect_effective_layout, EffectiveLayout};

/// Build a plugin widget lookup map from AppState.
///
/// Plugin renderers only see [`HostState`](xtop_plugin_api::HostState);
/// they keep precedence over every pack.
fn plugin_widgets(state: &AppState) -> HashMap<String, PluginWidgetFn> {
    let mut map: HashMap<String, PluginWidgetFn> = HashMap::new();
    for reg in &state.plugin_widgets {
        map.insert(reg.name.clone(), reg.render.clone());
    }
    map
}

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    if area.width < 40 || area.height < 8 {
        render_too_small(f, state, area);
        return;
    }

    if state.show_help {
        help::render(f, state, area);
        return;
    }

    if state.full_screen_widget != FullScreenWidget::None {
        render_fullscreen(f, state, area);
        return;
    }

    let mode = detect_effective_layout(area.width, area.height, state.layout_mode);
    let pw = plugin_widgets(state);

    if mode == EffectiveLayout::Minimal {
        render_minimal(f, state, area);
    } else {
        let def = state.current_layout();
        render_layout(f, state, area, def, &pw);
    }

    if state.input_mode == InputMode::Searching {
        render_search_overlay(f, state, area);
    } else if state.input_mode == InputMode::CommandPalette {
        palette::render(f, state, area);
    }
}

fn render_too_small(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::Paragraph;
    let fg = to_color(state.current_theme.fg());
    let text = Paragraph::new("Terminal too small\nMinimum: 40x8").style(Style::default().fg(fg));
    f.render_widget(text, area);
}

fn render_search_overlay(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let search_text = format!("/{}_", state.search_query);
    let overlay = Paragraph::new(search_text)
        .style(Style::default().fg(fg).bg(bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search Processes"),
        );
    let overlay_area = Rect {
        x: area.width.saturating_sub(40) / 2,
        y: area.height.saturating_sub(5) / 2,
        width: 40.min(area.width),
        height: 3.min(area.height),
    };
    f.render_widget(overlay, overlay_area);
}

fn render_fullscreen(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    let pw = plugin_widgets(state);
    render_named(f, state, "header", chunks[0], &pw);
    let name = fullscreen_widget_name(state.full_screen_widget);
    if !render_named(f, state, name, chunks[1], &pw) {
        let text = format!("No widget registered for '{name}'");
        let fg = to_color(state.current_theme.fg());
        let p = ratatui::widgets::Paragraph::new(text).style(Style::default().fg(fg));
        f.render_widget(p, chunks[1]);
    }
}

fn fullscreen_widget_name(w: FullScreenWidget) -> &'static str {
    match w {
        FullScreenWidget::Cpu => "cpu",
        FullScreenWidget::Memory => "memory",
        FullScreenWidget::Storage => "storage",
        FullScreenWidget::Network => "network",
        FullScreenWidget::Processes => "processes",
        FullScreenWidget::DiskIO => "disk_io",
        FullScreenWidget::Gpu => "gpu",
        FullScreenWidget::Battery => "battery",
        FullScreenWidget::None => "cpu",
    }
}

fn render_minimal(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::Gauge;

    let bg = to_color(state.current_theme.bg());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    let pw = plugin_widgets(state);
    render_named(f, state, "header", chunks[0], &pw);

    let Some(snap) = state.snapshot_cache() else {
        return;
    };
    let cpu_pct = snap.cpus.first().map(|c| c.usage).unwrap_or(0.0);
    let cpu_text = format!(
        "CPU: {:>3.0}%  |  Mem: {:.1}/{:.1}G ({:>3.0}%)",
        cpu_pct,
        snap.memory.used as f64 / 1073741824.0,
        snap.memory.total as f64 / 1073741824.0,
        snap.memory.percent,
    );
    let cpu_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(to_color(&state.current_theme.palette[1]))
                .bg(bg),
        )
        .percent(cpu_pct as u16)
        .label(cpu_text);
    f.render_widget(cpu_gauge, chunks[1]);

    let mem_pct = snap.memory.percent as u16;
    let mem_text = format!(
        "Mem: {:.1}/{:.1}G ({:>3.0}%)",
        snap.memory.used as f64 / 1073741824.0,
        snap.memory.total as f64 / 1073741824.0,
        snap.memory.percent,
    );
    let mem_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(to_color(&state.current_theme.palette[2]))
                .bg(bg),
        )
        .percent(mem_pct)
        .label(mem_text);
    f.render_widget(mem_gauge, chunks[2]);

    render_named(f, state, "processes", chunks[3], &pw);
}
