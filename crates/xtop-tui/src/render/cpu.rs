use crate::color::{gauge_gradient, to_color};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(state.current_theme.fg());
    let bg = to_color(state.current_theme.bg());
    let snap = state.snapshot();

    let title = if snap.cpu_temp > 0.0 {
        format!("CPU (Max: {:.1}°C)", snap.cpu_temp)
    } else {
        "CPU".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if snap.cpus.is_empty() {
        return;
    }

    let count = snap.cpus.len();
    let cols = if inner.width > 40 { 2 } else { 1 };
    let col_constraints = if cols == 2 {
        vec![Constraint::Percentage(50); 2]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let col_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(col_constraints)
        .split(inner);

    let per_col = count.div_ceil(cols);
    let chart_avail = inner.height > per_col as u16 + 4;

    // Render all core gauges
    for (col_idx, col_area) in col_areas.iter().enumerate() {
        let start = col_idx * per_col;
        let end = (start + per_col).min(count);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); end - start])
            .split(*col_area);

        for (i, row_area) in rows.iter().enumerate() {
            let cpu_idx = start + i;
            if cpu_idx >= count {
                break;
            }
            let cpu = &snap.cpus[cpu_idx];
            let usage = cpu.usage;
            let color_idx = if usage > state.alerts.cpu_high {
                1
            } else {
                gauge_gradient(usage, state.alerts.cpu_high)
            };
            let label = format!("CPU{:<2} {:>3.0}%", cpu.cpu_id, usage);
            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(to_color(&state.current_theme.palette[color_idx]))
                        .bg(bg),
                )
                .percent(usage as u16)
                .label(label);
            f.render_widget(gauge, *row_area);
        }
    }

    // Aggregate CPU chart below gauges
    if chart_avail {
        let gauge_height = per_col as u16; // min height for gauges in one column
        let chart_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(gauge_height), Constraint::Min(0)])
            .split(inner)
            .last()
            .copied()
            .unwrap_or(inner);

        let max_len = state.history.cpu.iter().map(|h| h.len()).max().unwrap_or(0);
        if max_len > 1 {
            let mut avg: Vec<(f64, f64)> = Vec::new();
            for tick in 0..max_len {
                let mut sum = 0.0;
                let mut n = 0;
                for core_hist in &state.history.cpu {
                    if tick < core_hist.len() {
                        sum += core_hist[tick].1;
                        n += 1;
                    }
                }
                if n > 0 {
                    let x = state.history.cpu[0]
                        .get(tick)
                        .map(|&(x, _)| x)
                        .unwrap_or(0.0);
                    avg.push((x, sum / n as f64));
                }
            }

            let datasets = vec![Dataset::default()
                .name("CPU Avg")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(to_color(&state.current_theme.palette[1])))
                .data(&avg)];

            let x_min = avg.first().map(|&(x, _)| x).unwrap_or(0.0);
            let x_max = avg.last().map(|&(x, _)| x).unwrap_or(100.0);
            let x_max = x_max.max(x_min + 1.0);

            let chart = Chart::new(datasets)
                .block(Block::default().borders(Borders::TOP))
                .x_axis(
                    Axis::default()
                        .bounds([x_min, x_max])
                        .labels(vec![Span::raw("")]),
                )
                .y_axis(
                    Axis::default()
                        .bounds([0.0, 100.0])
                        .labels(vec![
                            Span::raw("0%"),
                            Span::raw("50%"),
                            Span::raw("100%"),
                        ]),
                );
            f.render_widget(chart, chart_area);
        }
    }
}
