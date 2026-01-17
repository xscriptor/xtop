use crate::theme::{Theme, get_themes};
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind, Disks, Networks, ProcessRefreshKind, Components};
use std::collections::HashMap;
use ratatui::widgets::TableState;

#[allow(dead_code)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LayoutMode {
    Dashboard, // The current default split view
    Vertical,  // Everything stacked vertically
    ProcessFocus, // Focus mainly on processes
}

impl LayoutMode {
    pub fn next(&self) -> Self {
        match self {
            LayoutMode::Dashboard => LayoutMode::Vertical,
            LayoutMode::Vertical => LayoutMode::ProcessFocus,
            LayoutMode::ProcessFocus => LayoutMode::Dashboard,
        }
    }
}

pub struct App {
    pub sys: System,
    pub disks: Disks,
    pub networks: Networks,
    pub components: Components,
    pub themes: HashMap<String, Theme>,
    pub current_theme: Theme,
    pub should_quit: bool,
    pub theme_list: Vec<String>,
    pub selected_theme_index: usize,
    #[allow(dead_code)]
    pub show_help: bool,
    pub layout_mode: LayoutMode,
    
    // History Data for Charts
    pub cpu_history: Vec<Vec<(f64, f64)>>,
    pub mem_history: Vec<(f64, f64)>,
    #[allow(dead_code)]
    pub swap_history: Vec<(f64, f64)>,
    pub net_rx_history: Vec<(f64, f64)>,
    pub net_tx_history: Vec<(f64, f64)>,
    pub tick_count: f64,
    
    // UI States
    #[allow(dead_code)]
    pub process_table_state: TableState,
}

impl App {
    pub fn new() -> App {
        let themes = get_themes();
        let mut theme_list: Vec<String> = themes.keys().cloned().collect();
        theme_list.sort();
        
        let default_theme_name = "x";
        let current_theme = themes.get(default_theme_name).cloned().unwrap_or_else(|| {
             themes.values().next().unwrap().clone()
        });

        let selected_theme_index = theme_list.iter().position(|r| r == &current_theme.name).unwrap_or(0);

        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything())
        );
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        let mut process_table_state = TableState::default();
        process_table_state.select(Some(0));

        App {
            sys,
            disks,
            networks,
            components,
            themes,
            current_theme,
            should_quit: false,
            theme_list,
            selected_theme_index,
            show_help: false,
            layout_mode: LayoutMode::Dashboard,
            cpu_history: vec![],
            mem_history: vec![],
            swap_history: vec![],
            net_rx_history: vec![],
            net_tx_history: vec![],
            tick_count: 0.0,
            process_table_state,
        }
    }

    pub fn on_tick(&mut self) {
        self.sys.refresh_all();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);
        
        self.tick_count += 1.0;
        let x = self.tick_count;

        // Update CPU History
        let cpus = self.sys.cpus();
        if self.cpu_history.len() != cpus.len() {
            self.cpu_history = vec![vec![]; cpus.len()];
        }
        for (i, cpu) in cpus.iter().enumerate() {
            self.cpu_history[i].push((x, cpu.cpu_usage() as f64));
            if self.cpu_history[i].len() > 100 {
                self.cpu_history[i].remove(0);
            }
        }

        // Update Memory History
        let total_mem = self.sys.total_memory() as f64;
        let used_mem = self.sys.used_memory() as f64;
        let mem_usage = if total_mem > 0.0 { used_mem / total_mem * 100.0 } else { 0.0 };
        self.mem_history.push((x, mem_usage));
        if self.mem_history.len() > 100 {
            self.mem_history.remove(0);
        }

        // Update Net History
        let mut total_rx = 0;
        let mut total_tx = 0;
        for (_, network) in &self.networks {
            total_rx += network.received();
            total_tx += network.transmitted();
        }
        
        self.net_rx_history.push((x, total_rx as f64)); 
        self.net_tx_history.push((x, total_tx as f64));
         if self.net_rx_history.len() > 100 {
            self.net_rx_history.remove(0);
            self.net_tx_history.remove(0);
        }
    }

    pub fn next_theme(&mut self) {
        if self.selected_theme_index >= self.theme_list.len() - 1 {
            self.selected_theme_index = 0;
        } else {
            self.selected_theme_index += 1;
        }
        self.apply_theme();
    }

    pub fn previous_theme(&mut self) {
        if self.selected_theme_index == 0 {
            self.selected_theme_index = self.theme_list.len() - 1;
        } else {
            self.selected_theme_index -= 1;
        }
        self.apply_theme();
    }

    fn apply_theme(&mut self) {
        let theme_name = &self.theme_list[self.selected_theme_index];
        if let Some(theme) = self.themes.get(theme_name) {
            self.current_theme = theme.clone();
        }
    }
    
    pub fn next_layout(&mut self) {
        self.layout_mode = self.layout_mode.next();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
