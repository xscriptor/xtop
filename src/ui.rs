use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Gauge, Wrap, Table, Row, Cell, Chart, Dataset, Axis, GraphType},
    Frame,
};
use crate::app::{App, LayoutMode};
use sysinfo::{System, Process};

pub fn ui(f: &mut Frame, app: &App) {
    match app.layout_mode {
        LayoutMode::Dashboard => render_dashboard(f, app),
        LayoutMode::Vertical => render_vertical(f, app),
        LayoutMode::ProcessFocus => render_process_focus(f, app),
    }
}

fn render_dashboard(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Percentage(45), // Top half (CPU, Mem, Net)
            Constraint::Percentage(52), // Bottom half (Processes)
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_cpu(f, app, top_chunks[0]);
    
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33), // Memory
            Constraint::Percentage(33), // Storage
            Constraint::Percentage(34), // Network
        ])
        .split(top_chunks[1]);
        
    render_memory(f, app, right_chunks[0]);
    render_storage(f, app, right_chunks[1]);
    render_network(f, app, right_chunks[2]);

    render_processes(f, app, chunks[2]);
}

fn render_vertical(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(10), // CPU
            Constraint::Length(10), // Memory
            Constraint::Length(10), // Storage
            Constraint::Length(6),  // Network
            Constraint::Min(0),     // Processes
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_cpu(f, app, chunks[1]);
    render_memory(f, app, chunks[2]);
    render_storage(f, app, chunks[3]);
    render_network(f, app, chunks[4]);
    render_processes(f, app, chunks[5]);
}

fn render_process_focus(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(10), // Quick Stats Row
            Constraint::Min(0),     // Processes (Dominant)
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);

    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);
        
    render_cpu(f, app, stats_chunks[0]);
    render_memory(f, app, stats_chunks[1]);
    render_storage(f, app, stats_chunks[2]);
    render_network(f, app, stats_chunks[3]);
    
    render_processes(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let load_avg = System::load_average();
    let uptime = System::uptime();
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let minutes = (uptime % 3600) / 60;
    let seconds = uptime % 60;

    let mode_str = match app.layout_mode {
        LayoutMode::Dashboard => "Dashboard",
        LayoutMode::Vertical => "Vertical",
        LayoutMode::ProcessFocus => "Process Focus",
    };

    let text = format!(
        "xtop | Theme: {} | Layout: {} | Uptime: {}d {}h {}m {}s | Load: {:.2} {:.2} {:.2} | [q] Quit [t] Theme [l] Layout",
        app.current_theme.name, mode_str, days, hours, minutes, seconds, load_avg.one, load_avg.five, load_avg.fifteen
    );

    let p = Paragraph::new(text)
        .style(Style::default().fg(app.current_theme.fg()).bg(app.current_theme.bg()))
        .block(Block::default().borders(Borders::ALL).title("System Info"));
    f.render_widget(p, area);
}

fn render_cpu(f: &mut Frame, app: &App, area: Rect) {
    // Try to get max temp
    let mut max_temp = 0.0;
    for component in &app.components {
        if component.label().to_lowercase().contains("core") || component.label().to_lowercase().contains("cpu") {
            if let Some(temp) = component.temperature() {
                if temp > max_temp {
                    max_temp = temp;
                }
            }
        }
    }
    
    let title = if max_temp > 0.0 {
        format!("CPU (Max: {:.1}°C)", max_temp)
    } else {
        "CPU".to_string()
    };

    let block = Block::default().title(title).borders(Borders::ALL)
        .style(Style::default().fg(app.current_theme.fg()).bg(app.current_theme.bg()));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let cpus = app.sys.cpus();
    let count = cpus.len();
    if count == 0 { return; }

    let constraints = if inner_area.width > 40 {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    
    let cols = Layout::default().direction(Direction::Horizontal).constraints(constraints).split(inner_area);
    
    let items_per_col = (count + 1) / cols.len(); 
    
    for (col_idx, col_area) in cols.iter().enumerate() {
        let start = col_idx * items_per_col;
        let end = (start + items_per_col).min(count);
        
        let rows = Layout::default().direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); end - start])
            .split(*col_area);
            
        for (i, row_area) in rows.iter().enumerate() {
            let cpu_idx = start + i;
            if cpu_idx >= count { break; }
            
            let cpu = &cpus[cpu_idx];
            let usage = cpu.cpu_usage();
            let label = format!("CPU{:<2} {:>3.0}%", cpu_idx, usage);
            
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(app.current_theme.palette[1 + (cpu_idx % 6)]).bg(app.current_theme.bg()))
                .percent(usage as u16)
                .label(label);
            f.render_widget(gauge, *row_area);
        }
    }
}

fn render_memory(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Memory").borders(Borders::ALL)
        .style(Style::default().fg(app.current_theme.fg()).bg(app.current_theme.bg()));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
        .split(inner_area);

    // RAM Stats
    let total_mem = app.sys.total_memory();
    let used_mem = app.sys.used_memory();
    let free_mem = app.sys.free_memory();
    let available_mem = app.sys.available_memory();
    
    let mem_pct = if total_mem > 0 { (used_mem as f64 / total_mem as f64 * 100.0) as u16 } else { 0 };
    
    let mem_text = format!(
        "RAM: Total: {:.1} GB | Used: {:.1} GB | Avail: {:.1} GB", 
        total_mem as f64 / 1024.0 / 1024.0 / 1024.0,
        used_mem as f64 / 1024.0 / 1024.0 / 1024.0,
        available_mem as f64 / 1024.0 / 1024.0 / 1024.0
    );

    let mem_gauge = Gauge::default()
        .gauge_style(Style::default().fg(app.current_theme.palette[2]).bg(app.current_theme.bg()))
        .percent(mem_pct)
        .label(mem_text);
    f.render_widget(mem_gauge, chunks[0]);

    // Swap Stats
    let total_swap = app.sys.total_swap();
    let used_swap = app.sys.used_swap();
    let free_swap = app.sys.free_swap();
    let swap_pct = if total_swap > 0 { (used_swap as f64 / total_swap as f64 * 100.0) as u16 } else { 0 };
    
    let swap_text = format!(
        "SWP: Total: {:.1} GB | Used: {:.1} GB | Free: {:.1} GB", 
        total_swap as f64 / 1024.0 / 1024.0 / 1024.0,
        used_swap as f64 / 1024.0 / 1024.0 / 1024.0,
        free_swap as f64 / 1024.0 / 1024.0 / 1024.0
    );

    let swap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(app.current_theme.palette[3]).bg(app.current_theme.bg()))
        .percent(swap_pct)
        .label(swap_text);
    f.render_widget(swap_gauge, chunks[1]);
    
    // History Chart
    let datasets = vec![
        Dataset::default()
            .name("RAM Usage")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(app.current_theme.palette[2]))
            .data(&app.mem_history),
    ];
    
    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::TOP))
        .x_axis(Axis::default().bounds([app.tick_count - 100.0, app.tick_count]))
        .y_axis(Axis::default().bounds([0.0, 100.0]));
    f.render_widget(chart, chunks[2]);
}

fn render_storage(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Storage").borders(Borders::ALL)
        .style(Style::default().fg(app.current_theme.fg()).bg(app.current_theme.bg()));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // List all disks
    // We need to calculate how many lines per disk to fit them all, or just list them.
    let disk_count = app.disks.len();
    if disk_count == 0 { return; }

    let constraints = vec![Constraint::Length(3); disk_count];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);
        
    for (i, disk) in app.disks.iter().enumerate() {
        if i >= chunks.len() { break; }
        
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;
        let disk_pct = if total_space > 0 { (used_space as f64 / total_space as f64 * 100.0) as u16 } else { 0 };
        
        // "encima total uso y libre" -> We put stats in the label or above.
        // Let's format it nicely: "/mnt/data  Total: 100G  Used: 50G  Free: 50G"
        let label = format!(
            "{:?}  Tot: {:.0}G  Use: {:.0}G  Free: {:.0}G",
            disk.mount_point(),
            total_space as f64 / 1024.0 / 1024.0 / 1024.0,
            used_space as f64 / 1024.0 / 1024.0 / 1024.0,
            available_space as f64 / 1024.0 / 1024.0 / 1024.0
        );

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(app.current_theme.palette[4]).bg(app.current_theme.bg()))
            .percent(disk_pct)
            .label(label);
        f.render_widget(gauge, chunks[i]);
    }
}

fn render_network(f: &mut Frame, app: &App, area: Rect) {
     let block = Block::default().title("Network").borders(Borders::ALL)
        .style(Style::default().fg(app.current_theme.fg()).bg(app.current_theme.bg()));
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    let mut total_rx = 0;
    let mut total_tx = 0;
    for (_, network) in &app.networks {
        total_rx += network.received();
        total_tx += network.transmitted();
    }
    
    let text = vec![
        Line::from(vec![
            Span::styled("Total RX: ", Style::default().fg(app.current_theme.fg())),
            Span::styled(format!("{:.2} MB", total_rx as f64 / 1024.0 / 1024.0), Style::default().fg(app.current_theme.palette[4])),
        ]),
        Line::from(vec![
            Span::styled("Total TX: ", Style::default().fg(app.current_theme.fg())),
            Span::styled(format!("{:.2} MB", total_tx as f64 / 1024.0 / 1024.0), Style::default().fg(app.current_theme.palette[5])),
        ]),
    ];
    
    let p = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(p, inner_area);
}

fn render_processes(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Processes").borders(Borders::ALL)
        .style(Style::default().fg(app.current_theme.fg()).bg(app.current_theme.bg()));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let mut procs: Vec<&Process> = app.sys.processes().values().collect();
    procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));
    
    let procs = procs.iter().take(50);
    
    // Faint line separator
    let separator = Span::styled(" | ", Style::default().fg(app.current_theme.palette[8])); // Assuming palette[8] is faint/gray

    let rows: Vec<Row> = procs.map(|p| {
        Row::new(vec![
            Cell::from(Line::from(vec![Span::raw(p.pid().to_string()), separator.clone()])),
            Cell::from(Line::from(vec![Span::raw(p.name().to_string_lossy().into_owned()), separator.clone()])), 
            Cell::from(Line::from(vec![Span::raw(format!("{:.1}%", p.cpu_usage())), separator.clone()])),
            Cell::from(Line::from(vec![Span::raw(format!("{:.1} MB", p.memory() as f64 / 1024.0 / 1024.0)), separator.clone()])),
            Cell::from(p.user_id().map(|u| u.to_string()).unwrap_or_else(|| "?".to_string())),
        ])
        .style(Style::default().fg(app.current_theme.fg()))
    }).collect();

    let widths = [
        Constraint::Length(10), // Increased slightly for separator
        Constraint::Percentage(40),
        Constraint::Length(12),
        Constraint::Length(17),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["PID |", "Name |", "CPU% |", "Mem |", "User"])
                .style(Style::default().fg(app.current_theme.palette[6]).add_modifier(Modifier::BOLD))
                .bottom_margin(1)
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        
    f.render_widget(table, inner_area);
}
