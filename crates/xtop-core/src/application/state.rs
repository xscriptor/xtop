use crate::application::history::MetricsHistory;
use crate::domain::metrics::SystemSnapshot;
use crate::domain::system_info::SystemDataProvider;
use crate::domain::theme::Theme;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayoutMode {
    Dashboard,
    Vertical,
    Horizontal,
    CpuFocus,
    MemoryFocus,
    NetworkFocus,
    ProcessFocus,
}

impl LayoutMode {
    pub fn next(self) -> Self {
        match self {
            Self::Dashboard => Self::Vertical,
            Self::Vertical => Self::Horizontal,
            Self::Horizontal => Self::CpuFocus,
            Self::CpuFocus => Self::MemoryFocus,
            Self::MemoryFocus => Self::NetworkFocus,
            Self::NetworkFocus => Self::ProcessFocus,
            Self::ProcessFocus => Self::Dashboard,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Vertical => "Vertical",
            Self::Horizontal => "Horizontal",
            Self::CpuFocus => "CPU Focus",
            Self::MemoryFocus => "Memory Focus",
            Self::NetworkFocus => "Network Focus",
            Self::ProcessFocus => "Process Focus",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectiveLayout {
    Dashboard,
    Compact,
    Vertical,
    Horizontal,
    CpuFocus,
    MemoryFocus,
    NetworkFocus,
    ProcessFocus,
    Minimal,
}

pub fn detect_effective_layout(width: u16, height: u16, user_mode: LayoutMode) -> EffectiveLayout {
    if width < 60 || height < 14 {
        return EffectiveLayout::Minimal;
    }
    match user_mode {
        LayoutMode::Dashboard => {
            if width < 80 {
                EffectiveLayout::Vertical
            } else if width < 100 || height < 28 {
                EffectiveLayout::Compact
            } else {
                EffectiveLayout::Dashboard
            }
        }
        LayoutMode::Vertical => EffectiveLayout::Vertical,
        LayoutMode::Horizontal => EffectiveLayout::Horizontal,
        LayoutMode::CpuFocus => EffectiveLayout::CpuFocus,
        LayoutMode::MemoryFocus => EffectiveLayout::MemoryFocus,
        LayoutMode::NetworkFocus => EffectiveLayout::NetworkFocus,
        LayoutMode::ProcessFocus => EffectiveLayout::ProcessFocus,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FullScreenWidget {
    None,
    Cpu,
    Memory,
    Storage,
    Network,
    Processes,
    DiskIO,
    Gpu,
    Battery,
}

impl FullScreenWidget {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Cpu,
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Storage,
            Self::Storage => Self::Network,
            Self::Network => Self::Processes,
            Self::Processes => Self::DiskIO,
            Self::DiskIO => Self::Gpu,
            Self::Gpu => Self::Battery,
            Self::Battery => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Processes => "Processes",
            Self::DiskIO => "Disk I/O",
            Self::Gpu => "GPU",
            Self::Battery => "Battery",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Searching,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub cpu_high: f64,
    pub mem_high: f64,
    pub disk_high: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_high: 90.0,
            mem_high: 90.0,
            disk_high: 90.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub layout_mode: LayoutMode,
    pub update_interval_ms: u64,
    pub history_points: usize,
    pub alerts: AlertThresholds,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "x".to_string(),
            layout_mode: LayoutMode::Dashboard,
            update_interval_ms: 1000,
            history_points: 100,
            alerts: AlertThresholds::default(),
        }
    }
}

pub struct AppState {
    provider: Box<dyn SystemDataProvider>,
    pub history: MetricsHistory,
    pub should_quit: bool,
    pub layout_mode: LayoutMode,
    pub current_theme: Theme,
    pub themes: Vec<Theme>,
    pub selected_theme_index: usize,
    pub tick_count: f64,
    pub show_help: bool,
    pub input_mode: InputMode,
    pub search_query: String,
    pub full_screen_widget: FullScreenWidget,
    pub alerts: AlertThresholds,
    pub update_interval_ms: u64,
    pub config_path: String,
}

impl AppState {
    pub fn new(provider: Box<dyn SystemDataProvider>, themes: Vec<Theme>, config: Config) -> Self {
        let selected_theme_index = themes
            .iter()
            .position(|t| t.name == config.theme)
            .unwrap_or(0);
        let current_theme = themes[selected_theme_index].clone();
        Self {
            provider,
            history: MetricsHistory::new(config.history_points),
            should_quit: false,
            layout_mode: config.layout_mode,
            current_theme,
            themes,
            selected_theme_index,
            tick_count: 0.0,
            show_help: false,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            full_screen_widget: FullScreenWidget::None,
            alerts: config.alerts,
            update_interval_ms: config.update_interval_ms,
            config_path: String::new(),
        }
    }

    pub fn on_tick(&mut self) {
        self.provider.refresh_all();
        self.tick_count += 1.0;
        let x = self.tick_count;
        let snap = self.provider.snapshot();

        if snap.cpus.len() != self.history.cpu.len() {
            self.history.reset_cpu(snap.cpus.len());
        }
        for (i, cpu) in snap.cpus.iter().enumerate() {
            self.history.push_cpu(i, x, cpu.usage);
        }

        self.history.push_mem(x, snap.memory.percent);

        let total_rx: u64 = snap.networks.iter().map(|n| n.received).sum();
        let total_tx: u64 = snap.networks.iter().map(|n| n.transmitted).sum();
        self.history.push_net(x, total_rx as f64, total_tx as f64);
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        let mut snap = self.provider.snapshot();
        snap.disk_io = self.provider.disk_io();
        snap.batteries = self.provider.batteries();
        snap.gpus = self.provider.gpu_info();
        snap.dockers = self.provider.docker_info();
        snap
    }

    pub fn next_theme(&mut self) {
        self.selected_theme_index = (self.selected_theme_index + 1) % self.themes.len();
        self.apply_theme();
    }

    pub fn previous_theme(&mut self) {
        if self.selected_theme_index == 0 {
            self.selected_theme_index = self.themes.len() - 1;
        } else {
            self.selected_theme_index -= 1;
        }
        self.apply_theme();
    }

    fn apply_theme(&mut self) {
        self.current_theme = self.themes[self.selected_theme_index].clone();
    }

    pub fn next_layout(&mut self) {
        self.layout_mode = self.layout_mode.next();
        self.full_screen_widget = FullScreenWidget::None;
    }

    pub fn toggle_fullscreen(&mut self) {
        self.full_screen_widget = match self.full_screen_widget {
            FullScreenWidget::None => FullScreenWidget::Cpu,
            _ => FullScreenWidget::None,
        };
    }

    pub fn cycle_fullscreen_widget(&mut self) {
        self.full_screen_widget = self.full_screen_widget.next();
    }

    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Searching;
        self.search_query.clear();
    }

    pub fn search_push_char(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn search_pop_char(&mut self) {
        self.search_query.pop();
    }

    pub fn end_search(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_mode_next() {
        assert_eq!(LayoutMode::Dashboard.next(), LayoutMode::Vertical);
        assert_eq!(LayoutMode::Vertical.next(), LayoutMode::Horizontal);
        assert_eq!(LayoutMode::Horizontal.next(), LayoutMode::CpuFocus);
        assert_eq!(LayoutMode::CpuFocus.next(), LayoutMode::MemoryFocus);
        assert_eq!(LayoutMode::MemoryFocus.next(), LayoutMode::NetworkFocus);
        assert_eq!(LayoutMode::NetworkFocus.next(), LayoutMode::ProcessFocus);
        assert_eq!(LayoutMode::ProcessFocus.next(), LayoutMode::Dashboard);
    }

    #[test]
    fn test_detect_effective_layout_large() {
        assert_eq!(
            detect_effective_layout(120, 40, LayoutMode::Dashboard),
            EffectiveLayout::Dashboard
        );
    }

    #[test]
    fn test_detect_effective_layout_compact() {
        assert_eq!(
            detect_effective_layout(90, 30, LayoutMode::Dashboard),
            EffectiveLayout::Compact
        );
    }

    #[test]
    fn test_detect_effective_layout_narrow() {
        assert_eq!(
            detect_effective_layout(70, 30, LayoutMode::Dashboard),
            EffectiveLayout::Vertical
        );
    }

    #[test]
    fn test_detect_effective_layout_minimal() {
        assert_eq!(
            detect_effective_layout(50, 15, LayoutMode::Dashboard),
            EffectiveLayout::Minimal
        );
    }

    #[test]
    fn test_detect_effective_layout_focus_respected() {
        assert_eq!(
            detect_effective_layout(80, 30, LayoutMode::CpuFocus),
            EffectiveLayout::CpuFocus
        );
        assert_eq!(
            detect_effective_layout(80, 30, LayoutMode::NetworkFocus),
            EffectiveLayout::NetworkFocus
        );
    }

    #[test]
    fn test_detect_effective_layout_focus_downgrade() {
        assert_eq!(
            detect_effective_layout(50, 30, LayoutMode::CpuFocus),
            EffectiveLayout::Minimal
        );
    }

    #[test]
    fn test_fullscreen_widget_cycle() {
        assert_eq!(FullScreenWidget::None.next(), FullScreenWidget::Cpu);
        assert_eq!(FullScreenWidget::Battery.next(), FullScreenWidget::None);
    }

    #[test]
    fn test_layout_mode_label() {
        assert_eq!(LayoutMode::Dashboard.label(), "Dashboard");
        assert_eq!(LayoutMode::CpuFocus.label(), "CPU Focus");
        assert_eq!(LayoutMode::Horizontal.label(), "Horizontal");
    }

    #[test]
    fn test_alert_thresholds_default() {
        let a = AlertThresholds::default();
        assert_eq!(a.cpu_high, 90.0);
        assert_eq!(a.mem_high, 90.0);
        assert_eq!(a.disk_high, 90.0);
    }

    #[test]
    fn test_config_default() {
        let c = Config::default();
        assert_eq!(c.theme, "x");
        assert_eq!(c.layout_mode, LayoutMode::Dashboard);
        assert_eq!(c.update_interval_ms, 1000);
    }

    #[test]
    fn test_search_operations() {}
}
