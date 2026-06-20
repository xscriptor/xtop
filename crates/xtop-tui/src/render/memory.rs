use crate::color::{gauge_gradient, to_color};
use crate::format::format_bytes;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let snap = state.snapshot();

    let mem_alert = snap.memory.percent > state.alerts.mem_high;
    let mem_color_idx = if mem_alert {
        1
    } else {
        gauge_gradient(snap.memory.percent, state.alerts.mem_high)
    };

    let mut title = "Memory".to_string();
    if mem_alert {
        title = format!("Memory ⚠ {:.0}%", snap.memory.percent);
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let has_chart_area = inner.height > 7;
    if has_chart_area {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);

        render_ram_gauge(f, state, chunks[0], &snap, bg, mem_color_idx);
        render_swap_gauge(f, state, chunks[1], &snap, bg);
        render_chart(f, state, chunks[2], bg);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(inner);
        render_ram_gauge(f, state, chunks[0], &snap, bg, mem_color_idx);
        render_swap_gauge(f, state, chunks[1], &snap, bg);
    }
}

fn render_ram_gauge(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    snap: &xtop_core::domain::metrics::SystemSnapshot,
    bg: Color,
    color_idx: usize,
) {
    let mem_pct = snap.memory.percent as u16;
    let label = format!(
        "RAM: {} / {} ({:>3.0}%)",
        format_bytes(snap.memory.used),
        format_bytes(snap.memory.total),
        snap.memory.percent,
    );
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(to_color(&state.current_theme.palette[color_idx]))
                .bg(bg),
        )
        .percent(mem_pct)
        .label(label);
    f.render_widget(gauge, area);
}

fn render_swap_gauge(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    snap: &xtop_core::domain::metrics::SystemSnapshot,
    bg: Color,
) {
    let swap_pct = snap.swap.percent as u16;
    let color_idx = gauge_gradient(snap.swap.percent, state.alerts.mem_high);
    let label = format!(
        "SWP: {} / {} ({:>3.0}%)",
        format_bytes(snap.swap.used),
        format_bytes(snap.swap.total),
        snap.swap.percent,
    );
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(to_color(&state.current_theme.palette[color_idx]))
                .bg(bg),
        )
        .percent(swap_pct)
        .label(label);
    f.render_widget(gauge, area);
}

fn render_chart(f: &mut Frame, state: &AppState, area: Rect, _bg: Color) {
    let mem_data: Vec<(f64, f64)> = state.history.mem.iter().copied().collect();
    if mem_data.is_empty() {
        return;
    }

    let datasets = vec![Dataset::default()
        .name("RAM Usage")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(to_color(&state.current_theme.palette[2])))
        .data(&mem_data)];

    let x_min = mem_data.first().map(|&(x, _)| x).unwrap_or(0.0);
    let x_max = mem_data.last().map(|&(x, _)| x).unwrap_or(100.0);
    let x_max = x_max.max(x_min + 1.0);

    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::TOP))
        .x_axis(
            Axis::default()
                .bounds([x_min, x_max])
                .labels(vec![Span::raw("")]),
        )
        .y_axis(Axis::default().bounds([0.0, 100.0]).labels(vec![
            Span::raw("0%"),
            Span::raw("50%"),
            Span::raw("100%"),
        ]));
    f.render_widget(chart, area);
}
