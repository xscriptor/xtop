use crate::application::history::MetricsHistory;
use crate::application::plugin_manager::PluginManager;
use crate::domain::keybinding::{Action, Keybindings};
use crate::domain::metrics::SystemInfo;
use crate::domain::layout::LayoutDef;
use crate::domain::metrics::SystemSnapshot;
use crate::domain::plugin::WidgetRegistration;
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

fn layout_index_from_mode(mode: LayoutMode, defs: &[LayoutDef]) -> usize {
    let label = mode.label();
    defs.iter().position(|d| d.name == label).unwrap_or(0)
}

fn mode_from_layout_index(index: usize) -> LayoutMode {
    match index {
        0 => LayoutMode::Dashboard,
        1 => LayoutMode::Vertical,
        2 => LayoutMode::Horizontal,
        3 => LayoutMode::CpuFocus,
        4 => LayoutMode::MemoryFocus,
        5 => LayoutMode::NetworkFocus,
        6 => LayoutMode::ProcessFocus,
        _ => LayoutMode::Dashboard,
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProcessSortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

impl ProcessSortBy {
    pub fn next(self) -> Self {
        match self {
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Pid,
            Self::Pid => Self::Name,
            Self::Name => Self::Cpu,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU%",
            Self::Memory => "Mem",
            Self::Pid => "PID",
            Self::Name => "Name",
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct PaletteEntry {
    pub label: String,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PalettePage {
    Main,
    Themes,
    Layouts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaletteState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub entries: Vec<PaletteEntry>,
    pub filtered: Vec<usize>,
    pub page: PalettePage,
}

impl PaletteState {
    pub fn filtered_entries(&self) -> Vec<&PaletteEntry> {
        self.filtered.iter().map(|&i| &self.entries[i]).collect()
    }

    pub fn title(&self) -> &str {
        match self.page {
            PalettePage::Main => "Command Palette",
            PalettePage::Themes => "Select Theme",
            PalettePage::Layouts => "Select Layout",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Searching,
    CommandPalette,
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

fn default_layout_mode() -> LayoutMode {
    LayoutMode::Dashboard
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    #[serde(default = "default_layout_mode")]
    pub layout_mode: LayoutMode,
    /// Layout name for custom layouts beyond the 7 built-in LayoutMode variants.
    /// If non-empty, takes precedence over `layout_mode`.
    #[serde(default)]
    pub layout_name: String,
    pub update_interval_ms: u64,
    pub history_points: usize,
    pub alerts: AlertThresholds,
    #[serde(default)]
    pub keybindings: Keybindings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "x".to_string(),
            layout_mode: LayoutMode::Dashboard,
            layout_name: String::new(),
            update_interval_ms: 1000,
            history_points: 100,
            alerts: AlertThresholds::default(),
            keybindings: Keybindings::default(),
        }
    }
}

pub struct AppState {
    provider: Box<dyn SystemDataProvider>,
    pub history: MetricsHistory,
    pub should_quit: bool,
    pub layout_mode: LayoutMode,
    pub layout_index: usize,
    pub layout_defs: Vec<LayoutDef>,
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
    pub palette: PaletteState,
    pub keybindings: Keybindings,
    pub process_sort: ProcessSortBy,
    pub process_selected: Option<usize>,
    pub sys_info: SystemInfo,
    pub plugin_manager: Option<PluginManager>,
    pub plugin_widgets: Vec<WidgetRegistration>,
}

impl AppState {
    pub fn new(
        provider: Box<dyn SystemDataProvider>,
        themes: Vec<Theme>,
        config: Config,
        layout_defs: Vec<LayoutDef>,
    ) -> Self {
        let selected_theme_index = themes
            .iter()
            .position(|t| t.name == config.theme)
            .unwrap_or(0);
        let current_theme = themes[selected_theme_index].clone();
        let layout_index = if !config.layout_name.is_empty() {
            layout_defs
                .iter()
                .position(|l| l.name == config.layout_name)
                .unwrap_or_else(|| layout_index_from_mode(config.layout_mode, &layout_defs))
        } else {
            layout_index_from_mode(config.layout_mode, &layout_defs)
        };
        Self {
            provider,
            history: MetricsHistory::new(config.history_points),
            should_quit: false,
            layout_mode: config.layout_mode,
            layout_index,
            layout_defs,
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
            palette: PaletteState {
                open: false,
                query: String::new(),
                selected: 0,
                entries: Vec::new(),
                filtered: Vec::new(),
                page: PalettePage::Main,
            },
            keybindings: config.keybindings,
            process_sort: ProcessSortBy::Cpu,
            process_selected: None,
            sys_info: SystemInfo::default(),
            plugin_manager: None,
            plugin_widgets: Vec::new(),
        }
    }

    /// Set the plugin manager and inject extra data providers into the composite provider.
    /// Called once during initialization after all plugins are registered.
    pub fn init_plugins(
        &mut self,
        mgr: PluginManager,
        extra_providers: Vec<Box<dyn SystemDataProvider>>,
    ) {
        if !extra_providers.is_empty() {
            self.provider.add_extras(extra_providers);
        }
        self.plugin_manager = Some(mgr);
        self.refresh_plugin_widgets();
    }

    /// Collect plugin widgets into the state for the TUI renderer.
    pub fn refresh_plugin_widgets(&mut self) {
        if let Some(ref mgr) = self.plugin_manager {
            let registrations = mgr.collect_widgets();
            // Only accept if the plugin has RenderWidgets capability
            self.plugin_widgets = registrations;
        }
    }

    /// Kill a process by PID. Returns true if the signal was sent.
    pub fn kill_process_by_pid(&mut self, pid: u32) -> bool {
        self.provider.kill_process(pid)
    }

    /// Set alert thresholds.
    pub fn set_alert_thresholds(&mut self, cpu: f64, mem: f64, disk: f64) {
        self.alerts = AlertThresholds {
            cpu_high: cpu,
            mem_high: mem,
            disk_high: disk,
        };
    }

    /// Switch to a theme by name. Returns true if found.
    pub fn set_theme_by_name(&mut self, name: &str) -> bool {
        if let Some(idx) = self.themes.iter().position(|t| t.name == name) {
            self.selected_theme_index = idx;
            self.apply_theme();
            true
        } else {
            false
        }
    }

    /// Switch to a layout by name. Returns true if found.
    pub fn set_layout_by_name(&mut self, name: &str) -> bool {
        if let Some(idx) = self.layout_defs.iter().position(|l| l.name == name) {
            self.layout_index = idx;
            self.layout_mode = self.save_layout_mode();
            self.full_screen_widget = FullScreenWidget::None;
            true
        } else {
            false
        }
    }

    pub fn current_layout(&self) -> &LayoutDef {
        &self.layout_defs[self.layout_index]
    }

    pub fn save_layout_mode(&self) -> LayoutMode {
        mode_from_layout_index(self.layout_index)
    }

    /// Safely access the plugin manager with a closure.
    /// Ensures the plugin manager is always restored after the operation.
    /// NOTE: does NOT call refresh_plugin_widgets — the caller must do it if needed.
    pub fn with_plugin_manager_mut<R>(
        &mut self,
        f: impl FnOnce(&mut PluginManager, &mut Self) -> R,
    ) -> R {
        let mut mgr = self
            .plugin_manager
            .take()
            .expect("PluginManager not initialized");
        let result = f(&mut mgr, self);
        self.plugin_manager = Some(mgr);
        result
    }

    pub fn on_tick(&mut self) {
        self.provider.refresh_all();
        self.tick_count += 1.0;
        let info = self.provider.system_info();
        if !info.hostname.is_empty() {
            self.sys_info = info;
        }
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

        // Let plugins tick
        self.with_plugin_manager_mut(|mgr, this| {
            mgr.tick_all(this);
        });
        self.refresh_plugin_widgets();
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        let mut snap = self.provider.snapshot();
        snap.disk_io = self.provider.disk_io();
        snap.batteries = self.provider.batteries();
        snap.gpus = self.provider.gpu_info();
        snap.dockers = self.provider.docker_info();
        snap.sys_info = self.provider.system_info();
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
        self.layout_index = (self.layout_index + 1) % self.layout_defs.len();
        self.layout_mode = self.save_layout_mode();
        self.full_screen_widget = FullScreenWidget::None;
    }

    pub fn current_layout_name(&self) -> &str {
        &self.layout_defs[self.layout_index].name
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

    pub fn rebuild_palette(&mut self) {
        self.palette.entries.clear();
        match self.palette.page {
            PalettePage::Main => {
                self.palette.entries.push(PaletteEntry {
                    label: "Themes →".into(),
                    action: Action::NavigateThemes,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Layouts →".into(),
                    action: Action::NavigateLayouts,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Toggle Fullscreen".into(),
                    action: Action::ToggleFullscreen,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Cycle Fullscreen Widget".into(),
                    action: Action::CycleFullscreen,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Search Processes".into(),
                    action: Action::Search,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Toggle Help".into(),
                    action: Action::ToggleHelp,
                });
                self.palette.entries.push(PaletteEntry {
                    label: format!("Sort: {}", self.process_sort.label()),
                    action: Action::SortByCpu,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Random Theme".into(),
                    action: Action::RandomTheme,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Exit".into(),
                    action: Action::Quit,
                });
            }
            PalettePage::Themes => {
                for (i, theme) in self.themes.iter().enumerate() {
                    self.palette.entries.push(PaletteEntry {
                        label: theme.name.clone(),
                        action: Action::SelectTheme(i),
                    });
                }
            }
            PalettePage::Layouts => {
                for (i, layout) in self.layout_defs.iter().enumerate() {
                    self.palette.entries.push(PaletteEntry {
                        label: layout.name.clone(),
                        action: Action::SelectLayout(i),
                    });
                }
            }
        }
        self.palette_filter();
    }

    pub fn open_palette(&mut self) {
        self.palette.open = true;
        self.palette.query.clear();
        self.palette.selected = 0;
        self.palette.page = PalettePage::Main;
        self.rebuild_palette();
    }

    pub fn palette_navigate_to(&mut self, page: PalettePage) {
        self.palette.page = page;
        self.palette.query.clear();
        self.palette.selected = 0;
        self.rebuild_palette();
    }

    pub fn palette_filter(&mut self) {
        let q = self.palette.query.to_lowercase();
        self.palette.filtered = self
            .palette
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| q.is_empty() || e.label.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        if !self.palette.filtered.is_empty() {
            self.palette.selected = self.palette.selected.min(self.palette.filtered.len() - 1);
        } else {
            self.palette.selected = 0;
        }
    }

    pub fn palette_select_next(&mut self) {
        if !self.palette.filtered.is_empty() {
            self.palette.selected = (self.palette.selected + 1) % self.palette.filtered.len();
        }
    }

    pub fn palette_select_prev(&mut self) {
        if !self.palette.filtered.is_empty() {
            self.palette.selected = if self.palette.selected == 0 {
                self.palette.filtered.len() - 1
            } else {
                self.palette.selected - 1
            };
        }
    }

    pub fn process_select_next(&mut self) {
        let snap = self.snapshot();
        if snap.processes.is_empty() {
            return;
        }
        let idx = self.process_selected.unwrap_or(0);
        self.process_selected = Some((idx + 1) % snap.processes.len());
    }

    pub fn process_select_prev(&mut self) {
        let snap = self.snapshot();
        if snap.processes.is_empty() {
            return;
        }
        let idx = self.process_selected.unwrap_or(0);
        self.process_selected = Some(if idx == 0 {
            snap.processes.len() - 1
        } else {
            idx - 1
        });
    }

    pub fn cycle_sort(&mut self) {
        self.process_sort = self.process_sort.next();
        self.process_selected = None;
    }

    pub fn palette_selected_action(&self) -> Option<Action> {
        self.palette
            .filtered
            .get(self.palette.selected)
            .and_then(|&i| self.palette.entries.get(i))
            .map(|e| e.action.clone())
    }

    pub fn close_palette(&mut self) {
        self.palette.open = false;
        self.palette.page = PalettePage::Main;
        self.input_mode = InputMode::Normal;
    }

    pub fn execute_action(&mut self, action: &Action) {
        match action {
            Action::Quit => self.quit(),
            Action::ToggleHelp => self.toggle_help(),
            Action::NextTheme => self.next_theme(),
            Action::PreviousTheme => self.previous_theme(),
            Action::NextLayout => self.next_layout(),
            Action::ToggleFullscreen => self.toggle_fullscreen(),
            Action::CycleFullscreen => self.cycle_fullscreen_widget(),
            Action::Search => self.start_search(),
            Action::OpenCommandPalette => {}
            Action::Cancel => {
                if self.show_help {
                    self.toggle_help();
                }
            }
            Action::SelectTheme(i) => {
                self.selected_theme_index = *i;
                self.apply_theme();
            }
            Action::SelectLayout(i) => {
                self.layout_index = *i;
                self.layout_mode = self.save_layout_mode();
                self.full_screen_widget = FullScreenWidget::None;
            }
            Action::NavigateThemes => {
                self.palette_navigate_to(PalettePage::Themes);
                return;
            }
            Action::NavigateLayouts => {
                self.palette_navigate_to(PalettePage::Layouts);
                return;
            }
            Action::KillProcess => {
                if let Some(pid) = self.process_selected {
                    let snap = self.snapshot();
                    if pid < snap.processes.len() {
                        let target = snap.processes[pid].pid;
                        self.provider.kill_process(target);
                        self.process_selected = None;
                    }
                }
            }
            Action::ProcessUp => self.process_select_prev(),
            Action::ProcessDown => self.process_select_next(),
            Action::SortByPid | Action::SortByCpu | Action::SortByName | Action::SortByMem => {
                self.cycle_sort();
            }
            Action::RandomTheme => {
                let n = self.themes.len();
                if n > 1 {
                    let next = (self.selected_theme_index + 7) % n;
                    self.selected_theme_index = next;
                    self.apply_theme();
                }
            }
        }
        // Close palette after executing any action (except navigation which returns above)
        if self.input_mode == InputMode::CommandPalette {
            self.close_palette();
        }
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
