//! Network widget: RX/TX rates per interface.

use crate::state::AppState;
use crate::ui::share::format_bytes;
use crate::ui::share::to_color;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());

    let block = Block::default()
        .title("Network")
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    let total_rx: u64 = snap.networks.iter().map(|n| n.received).sum();
    let total_tx: u64 = snap.networks.iter().map(|n| n.transmitted).sum();
    let total_rx_speed: f64 = snap.networks.iter().map(|n| n.rx_speed).sum();
    let total_tx_speed: f64 = snap.networks.iter().map(|n| n.tx_speed).sum();

    let has_chart = inner.height > 6;

    if has_chart {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(inner);

        render_stats(
            f,
            state,
            chunks[0],
            fg,
            total_rx,
            total_tx,
            total_rx_speed,
            total_tx_speed,
            &snap.networks,
        );
        render_net_chart(f, state, chunks[1], bg);
    } else {
        render_stats(
            f,
            state,
            inner,
            fg,
            total_rx,
            total_tx,
            total_rx_speed,
            total_tx_speed,
            &snap.networks,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_stats(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    fg: Color,
    total_rx: u64,
    total_tx: u64,
    total_rx_speed: f64,
    total_tx_speed: f64,
    interfaces: &[xtop_plugin_api::model::NetworkInfo],
) {
    let mut text = vec![
        Line::from(vec![
            Span::styled("RX: ", Style::default().fg(fg)),
            Span::styled(
                format_bytes(total_rx),
                Style::default().fg(to_color(&state.current_theme.palette[4])),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}/s", format_bytes(total_rx_speed as u64)),
                Style::default().fg(to_color(&state.current_theme.palette[4])),
            ),
        ]),
        Line::from(vec![
            Span::styled("TX: ", Style::default().fg(fg)),
            Span::styled(
                format_bytes(total_tx),
                Style::default().fg(to_color(&state.current_theme.palette[5])),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}/s", format_bytes(total_tx_speed as u64)),
                Style::default().fg(to_color(&state.current_theme.palette[5])),
            ),
        ]),
    ];

    if area.height > 4 {
        for iface in interfaces {
            if text.len() as u16 >= area.height.saturating_sub(1) {
                break;
            }
            text.push(Line::from(Span::raw(format!(
                " {}  RX: {}  TX: {}",
                iface.name,
                format_bytes(iface.received),
                format_bytes(iface.transmitted),
            ))));
        }
    }

    let p = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_net_chart(f: &mut Frame, state: &AppState, area: Rect, _bg: Color) {
    let rx_data: Vec<(f64, f64)> = state.history.net_rx.iter().copied().collect();
    let tx_data: Vec<(f64, f64)> = state.history.net_tx.iter().copied().collect();
    if rx_data.len() < 2 || tx_data.len() < 2 {
        return;
    }

    // Find max value for y-axis bounds
    let max_val = rx_data
        .iter()
        .chain(tx_data.iter())
        .map(|&(_, v)| v)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let datasets = vec![
        Dataset::default()
            .name("RX")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(to_color(&state.current_theme.palette[4])))
            .data(&rx_data),
        Dataset::default()
            .name("TX")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(to_color(&state.current_theme.palette[5])))
            .data(&tx_data),
    ];

    let x_min = rx_data.first().map(|&(x, _)| x).unwrap_or(0.0);
    let x_max = rx_data.last().map(|&(x, _)| x).unwrap_or(100.0);
    let x_max = x_max.max(x_min + 1.0);

    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::TOP))
        .x_axis(
            Axis::default()
                .bounds([x_min, x_max])
                .labels(vec![Span::raw("")]),
        )
        .y_axis(Axis::default().bounds([0.0, max_val]).labels(vec![
            Span::raw("0"),
            Span::raw(format!("{:.0}", max_val / 2.0)),
            Span::raw(format!("{:.0}", max_val)),
        ]));
    f.render_widget(chart, area);
}
