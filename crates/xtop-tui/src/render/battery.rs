use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Frame;
use xtop_core::application::state::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());

    let block = Block::default()
        .title("Battery")
        .borders(Borders::ALL)
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let snap = state.snapshot();
    if snap.batteries.is_empty() {
        let msg = Paragraph::new("No battery data available")
            .style(Style::default().fg(fg))
            .wrap(Wrap { trim: true });
        f.render_widget(msg, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); snap.batteries.len()])
        .split(inner);

    for (i, bat) in snap.batteries.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        let time_info = match (bat.time_to_full, bat.time_to_empty) {
            (Some(t), _) if bat.state == "Charging" => {
                format!(" {}m to full", t / 60)
            }
            (_, Some(t)) if bat.state == "Discharging" => {
                format!(" {}m remaining", t / 60)
            }
            _ => String::new(),
        };
        let label = format!(
            "{}  {:>3.0}%  {} {}",
            bat.name, bat.percentage, bat.state, time_info,
        );
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(rgb(&state.current_theme.palette[2]))
                    .bg(bg),
            )
            .percent(bat.percentage as u16)
            .label(label);
        f.render_widget(gauge, chunks[i]);
    }
}
