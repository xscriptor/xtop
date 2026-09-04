//! Live application state: system sampling per tick, layout/theme/process
//! control, plugins.

use crate::config::keybinding::{Action, Keybindings};
use crate::config::{Config, UiStyle};
use crate::plugins::PluginManager;
use crate::state::history::MetricsHistory;
use crate::state::view::{FullScreenWidget, InputMode, PalettePage, PaletteState, ProcessSortBy};
use crate::theme::Theme;
use xtop_layout::{layout_index_from_mode, layout_mode_for_name, LayoutDef, LayoutMode};
use xtop_plugin_api::model::{ProcessInfo, SystemInfo, SystemSnapshot};
use xtop_plugin_api::{AlertThresholds, PluginWidget, SystemDataProvider};

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
    /// Widget glyph style (chart charset, borders); from the user config.
    pub style: UiStyle,
    pub palette: PaletteState,
    pub keybindings: Keybindings,
    pub process_sort: ProcessSortBy,
    /// Selected process anchored by PID (not row index), so sorting/filtering
    /// or a fresh sample never makes a kill target the wrong process.
    pub process_selected_pid: Option<u32>,
    pub sys_info: SystemInfo,
    /// Latest full system sample, computed once per tick and shared by every
    /// widget/action in that frame (avoids N samples per frame).
    last_snapshot: Option<SystemSnapshot>,
    pub plugin_manager: Option<PluginManager>,
    pub plugin_widgets: Vec<PluginWidget>,
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
            style: config.style,
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
            process_selected_pid: None,
            sys_info: SystemInfo::default(),
            last_snapshot: None,
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
            self.sync_layout_mode();
            self.full_screen_widget = FullScreenWidget::None;
            true
        } else {
            false
        }
    }

    pub fn current_layout(&self) -> &LayoutDef {
        &self.layout_defs[self.layout_index]
    }

    /// The layout mode matching the current definition. Custom layouts fall
    /// back to the previously active mode (they are addressed by name).
    pub fn save_layout_mode(&self) -> LayoutMode {
        let fallback = if self.layout_index < 7 {
            xtop_layout::mode_from_layout_index(self.layout_index)
        } else {
            self.layout_mode
        };
        match self.layout_defs.get(self.layout_index) {
            Some(def) => layout_mode_for_name(&def.name, fallback),
            None => LayoutMode::Dashboard,
        }
    }

    fn sync_layout_mode(&mut self) {
        self.layout_mode = self.save_layout_mode();
    }

    /// Safely access the plugin manager with a closure.
    /// Ensures the plugin manager is always restored after the operation.
    /// Returns `None` when no manager is initialized (pre-bootstrap or tests)
    /// instead of panicking; callers decide how to degrade.
    /// NOTE: does NOT call refresh_plugin_widgets — the caller must do it if needed.
    pub fn with_plugin_manager_mut<R>(
        &mut self,
        f: impl FnOnce(&mut PluginManager, &mut Self) -> R,
    ) -> Option<R> {
        let mut mgr = self.plugin_manager.take()?;
        let result = f(&mut mgr, self);
        self.plugin_manager = Some(mgr);
        Some(result)
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

        // Network history tracks *rates* (bytes/s), not cumulative counters,
        // so the chart shows throughput over time.
        let total_rx_speed: f64 = snap.networks.iter().map(|n| n.rx_speed).sum();
        let total_tx_speed: f64 = snap.networks.iter().map(|n| n.tx_speed).sum();
        self.history.push_net(x, total_rx_speed, total_tx_speed);

        // Cache the sample for every widget and action in this frame.
        self.last_snapshot = Some(snap);

        // Let plugins tick
        let _ = self.with_plugin_manager_mut(|mgr, this| {
            mgr.tick_all(this);
        });
        self.refresh_plugin_widgets();
    }

    /// The current sample (one per tick). Widgets and process actions read
    /// this instead of resampling the system every frame.
    pub fn snapshot_cache(&self) -> Option<&SystemSnapshot> {
        self.last_snapshot.as_ref()
    }

    /// Full current snapshot. Prefer [`AppState::snapshot_cache`] in render
    /// paths; this forces a fresh system sample (used by plugin hosts).
    pub fn snapshot(&self) -> SystemSnapshot {
        self.last_snapshot
            .clone()
            .unwrap_or_else(|| self.provider.snapshot())
    }

    /// The process rows the UI shows: search filter + user sort, applied to
    /// one shared sample. Single source of truth for the processes widget and
    /// the Up/Down/Kill actions (selection is anchored by PID).
    pub fn sorted_processes<'a>(&'a self, snap: &'a SystemSnapshot) -> Vec<&'a ProcessInfo> {
        let mut items: Vec<&ProcessInfo> = if self.search_query.is_empty() {
            snap.processes.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            snap.processes
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&q))
                .collect()
        };
        match self.process_sort {
            ProcessSortBy::Cpu => items.sort_by(|a, b| {
                b.cpu_usage
                    .partial_cmp(&a.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            ProcessSortBy::Memory => items.sort_by_key(|b| std::cmp::Reverse(b.memory)),
            ProcessSortBy::Pid => items.sort_by_key(|a| a.pid),
            ProcessSortBy::Name => items.sort_by_key(|a| a.name.to_lowercase()),
        }
        items
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
        if self.layout_defs.is_empty() {
            return;
        }
        self.layout_index = (self.layout_index + 1) % self.layout_defs.len();
        self.sync_layout_mode();
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

    pub fn process_select_next(&mut self) {
        self.move_process_selection(1);
    }

    pub fn process_select_prev(&mut self) {
        self.move_process_selection(-1);
    }

    fn move_process_selection(&mut self, dir: i32) {
        let Some(snap) = self.snapshot_cache() else {
            return;
        };
        let view = self.sorted_processes(snap);
        if view.is_empty() {
            return;
        }
        let n = view.len() as i32;
        let pos = self
            .process_selected_pid
            .and_then(|pid| view.iter().position(|p| p.pid == pid))
            .unwrap_or(0) as i32;
        let next = (pos + dir).rem_euclid(n);
        self.process_selected_pid = Some(view[next as usize].pid);
    }

    pub fn cycle_sort(&mut self) {
        self.process_sort = self.process_sort.next();
        self.process_selected_pid = None;
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
                if *i < self.layout_defs.len() {
                    self.layout_index = *i;
                    self.sync_layout_mode();
                    self.full_screen_widget = FullScreenWidget::None;
                }
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
                if let Some(pid) = self.process_selected_pid {
                    self.provider.kill_process(pid);
                    self.process_selected_pid = None;
                }
            }
            Action::ProcessUp => self.process_select_prev(),
            Action::ProcessDown => self.process_select_next(),
            Action::SortByCpu => {
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
    use crate::config::default_alerts;
    use xtop_layout::{default_layouts, LayoutMode};

    fn test_state(defs: Vec<LayoutDef>) -> AppState {
        let theme = Theme {
            name: "test".into(),
            palette: [[0, 0, 0]; 16],
        };
        AppState::new(
            Box::new(crate::providers::sysinfo::SysinfoProvider::new()),
            vec![theme],
            Config::default(),
            defs,
        )
    }

    #[test]
    fn test_fullscreen_widget_cycle() {
        assert_eq!(FullScreenWidget::None.next(), FullScreenWidget::Cpu);
        assert_eq!(FullScreenWidget::Battery.next(), FullScreenWidget::None);
    }

    #[test]
    fn test_alert_thresholds_default() {
        let a = default_alerts();
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
    fn test_snapshot_cache_empty_before_first_tick() {
        let state = test_state(vec![]);
        assert!(state.snapshot_cache().is_none());
    }

    #[test]
    fn test_custom_layout_keeps_previous_mode() {
        let mut defs = default_layouts();
        defs.push(LayoutDef {
            name: "My Custom".into(),
            root: xtop_layout::LayoutNode::Split {
                direction: xtop_layout::Direction::Vertical,
                areas: vec![],
            },
        });
        let mut state = test_state(defs.clone());

        // Default boot: Dashboard at index 0.
        assert_eq!(state.layout_index, 0);
        assert_eq!(state.layout_mode, LayoutMode::Dashboard);

        // Custom layout (index 7) must not reset the mode to Dashboard-as-mode
        // loss; it falls back to the previously active mode (Dashboard here).
        assert!(state.set_layout_by_name("My Custom"));
        assert_eq!(state.layout_mode, LayoutMode::Dashboard);
        assert_eq!(state.current_layout_name(), "My Custom");

        // Built-ins still resolve to their real mode by name.
        assert!(state.set_layout_by_name("CPU Focus"));
        assert_eq!(state.layout_mode, LayoutMode::CpuFocus);
    }
}
