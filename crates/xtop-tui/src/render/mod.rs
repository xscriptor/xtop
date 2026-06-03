mod battery;
mod cpu;
mod disk_io;
mod gpu;
mod header;
mod help;
mod memory;
mod network;
mod processes;
mod storage;

use ratatui::prelude::*;
use ratatui::Frame;
use xtop_core::application::state::{
    detect_effective_layout, AppState, EffectiveLayout, FullScreenWidget, InputMode,
};

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
    match mode {
        EffectiveLayout::Dashboard => render_dashboard(f, state, area),
        EffectiveLayout::Compact => render_compact(f, state, area),
        EffectiveLayout::Vertical => render_vertical(f, state, area),
        EffectiveLayout::Horizontal => render_horizontal(f, state, area),
        EffectiveLayout::CpuFocus => render_cpu_focus(f, state, area),
        EffectiveLayout::MemoryFocus => render_memory_focus(f, state, area),
        EffectiveLayout::NetworkFocus => render_network_focus(f, state, area),
        EffectiveLayout::ProcessFocus => render_process_focus(f, state, area),
        EffectiveLayout::Minimal => render_minimal(f, state, area),
    }

    if state.input_mode == InputMode::Searching {
        render_search_overlay(f, state, area);
    }
}

fn render_too_small(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::Paragraph;
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let text = Paragraph::new("Terminal too small\nMinimum: 40x8").style(Style::default().fg(fg));
    f.render_widget(text, area);
}

fn render_search_overlay(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let fg = rgb(state.current_theme.fg());
    let bg = rgb(state.current_theme.bg());
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
    header::render(f, state, chunks[0]);
    match state.full_screen_widget {
        FullScreenWidget::Cpu => cpu::render(f, state, chunks[1]),
        FullScreenWidget::Memory => memory::render(f, state, chunks[1]),
        FullScreenWidget::Storage => storage::render(f, state, chunks[1]),
        FullScreenWidget::Network => network::render(f, state, chunks[1]),
        FullScreenWidget::Processes => processes::render(f, state, chunks[1]),
        FullScreenWidget::DiskIO => disk_io::render(f, state, chunks[1]),
        FullScreenWidget::Gpu => gpu::render(f, state, chunks[1]),
        FullScreenWidget::Battery => battery::render(f, state, chunks[1]),
        FullScreenWidget::None => {}
    }
}

fn render_dashboard(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(45),
            Constraint::Percentage(52),
        ])
        .split(area);

    header::render(f, state, chunks[0]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    cpu::render(f, state, top[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(top[1]);

    memory::render(f, state, right[0]);
    storage::render(f, state, right[1]);
    network::render(f, state, right[2]);

    processes::render(f, state, chunks[2]);
}

fn render_compact(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(47),
        ])
        .split(area);

    header::render(f, state, chunks[0]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    cpu::render(f, state, top[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(top[1]);

    memory::render(f, state, right[0]);
    storage::render(f, state, right[1]);
    network::render(f, state, right[2]);

    processes::render(f, state, chunks[2]);
}

fn render_vertical(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    header::render(f, state, chunks[0]);
    cpu::render(f, state, chunks[1]);
    memory::render(f, state, chunks[2]);
    storage::render(f, state, chunks[3]);
    network::render(f, state, chunks[4]);
    processes::render(f, state, chunks[5]);
}

fn render_horizontal(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    header::render(f, state, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    cpu::render(f, state, mid[0]);
    memory::render(f, state, mid[1]);
    storage::render(f, state, mid[2]);
    network::render(f, state, mid[3]);
}

fn render_cpu_focus(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(60),
            Constraint::Min(10),
        ])
        .split(area);

    header::render(f, state, chunks[0]);
    cpu::render(f, state, chunks[1]);
    processes::render(f, state, chunks[2]);
}

fn render_memory_focus(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(60),
            Constraint::Min(10),
        ])
        .split(area);

    header::render(f, state, chunks[0]);
    memory::render(f, state, chunks[1]);
    processes::render(f, state, chunks[2]);
}

fn render_network_focus(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Min(10),
        ])
        .split(area);

    header::render(f, state, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    network::render(f, state, mid[0]);
    disk_io::render(f, state, mid[1]);

    processes::render(f, state, chunks[2]);
}

fn render_process_focus(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    header::render(f, state, chunks[0]);

    let stats = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    cpu::render(f, state, stats[0]);
    memory::render(f, state, stats[1]);
    storage::render(f, state, stats[2]);
    network::render(f, state, stats[3]);

    processes::render(f, state, chunks[2]);
}

fn render_minimal(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::Gauge;
    let rgb = |c: &[u8; 3]| Color::Rgb(c[0], c[1], c[2]);
    let bg = rgb(state.current_theme.bg());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    header::render(f, state, chunks[0]);

    let snap = state.snapshot();
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
                .fg(rgb(&state.current_theme.palette[1]))
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
                .fg(rgb(&state.current_theme.palette[2]))
                .bg(bg),
        )
        .percent(mem_pct)
        .label(mem_text);
    f.render_widget(mem_gauge, chunks[2]);

    processes::render(f, state, chunks[3]);
}
