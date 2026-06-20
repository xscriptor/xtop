use std::fmt::Debug;

use crate::application::state::AppState;
use crate::domain::metrics::SystemSnapshot;
use crate::domain::system_info::SystemDataProvider;

/// Unique identifier for a plugin capability.
/// Used for permission checking and manifest declaration.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum PluginCapability {
    /// Read system metrics (CPU, memory, network, disks, processes)
    ReadSystemInfo,
    /// Terminate processes
    KillProcesses,
    /// Modify configuration (themes, layouts, alerts, interval)
    ModifyConfig,
    /// Register custom widgets in the TUI
    RenderWidgets,
    /// Anything not covered above
    Custom(String),
}

/// Static metadata about a plugin.
/// Returned by [`Plugin::manifest`].
#[derive(Clone, Debug)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
}

/// Error type for plugin operations.
#[derive(Debug)]
pub enum PluginError {
    /// A recoverable error (e.g. invalid params, resource busy)
    Recoverable(String),
    /// A fatal error (plugin should be disabled)
    Fatal(String),
    /// Action not understood by this plugin
    UnknownAction(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recoverable(msg) => write!(f, "{msg}"),
            Self::Fatal(msg) => write!(f, "FATAL: {msg}"),
            Self::UnknownAction(action) => write!(f, "unknown action: {action}"),
        }
    }
}

impl std::error::Error for PluginError {}

type RenderFn =
    std::sync::Arc<dyn Fn(&mut ratatui::Frame, &AppState, ratatui::prelude::Rect) + Send + Sync>;

/// A widget that a plugin registers for rendering in the TUI.
pub struct WidgetRegistration {
    pub name: String,
    pub render: RenderFn,
}

impl Debug for WidgetRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetRegistration")
            .field("name", &self.name)
            .finish()
    }
}

/// Context passed to plugin lifecycle methods.
/// Provides safe, limited access to application state and plugin-specific directories.
pub struct PluginContext<'a> {
    pub(crate) state: &'a mut AppState,
    pub(crate) plugin_data_dir: std::path::PathBuf,
    pub(crate) capabilities: Vec<PluginCapability>,
}

impl PluginContext<'_> {
    fn check_capability(&self, cap: &PluginCapability) -> Result<(), PluginError> {
        if self.capabilities.contains(cap) {
            Ok(())
        } else {
            Err(PluginError::Recoverable(format!(
                "plugin does not have required capability: {:?}",
                cap
            )))
        }
    }

    /// Full system snapshot with all available metrics.
    /// Requires `ReadSystemInfo` capability.
    pub fn snapshot(&self) -> SystemSnapshot {
        self.state.snapshot()
    }

    /// The top N processes sorted by CPU usage.
    /// Requires `ReadSystemInfo` capability.
    pub fn top_processes(&self, n: usize) -> Vec<crate::domain::metrics::ProcessInfo> {
        let snap = self.snapshot();
        snap.processes.into_iter().take(n).collect()
    }

    /// Kill a process by PID. Returns true if the signal was sent.
    /// Requires `KillProcesses` capability.
    pub fn kill_process(&mut self, pid: u32) -> Result<bool, PluginError> {
        self.check_capability(&PluginCapability::KillProcesses)?;
        Ok(self.state.kill_process_by_pid(pid))
    }

    /// Set alert thresholds for CPU, memory, and disk.
    /// Requires `ModifyConfig` capability.
    pub fn set_alert_thresholds(
        &mut self,
        cpu: f64,
        mem: f64,
        disk: f64,
    ) -> Result<(), PluginError> {
        self.check_capability(&PluginCapability::ModifyConfig)?;
        self.state.set_alert_thresholds(cpu, mem, disk);
        Ok(())
    }

    /// Switch to a theme by name. Returns true if found.
    /// Requires `ModifyConfig` capability.
    pub fn set_theme_by_name(&mut self, name: &str) -> Result<bool, PluginError> {
        self.check_capability(&PluginCapability::ModifyConfig)?;
        Ok(self.state.set_theme_by_name(name))
    }

    /// Switch to a layout by name. Returns true if found.
    /// Requires `ModifyConfig` capability.
    pub fn set_layout_by_name(&mut self, name: &str) -> Result<bool, PluginError> {
        self.check_capability(&PluginCapability::ModifyConfig)?;
        Ok(self.state.set_layout_by_name(name))
    }

    /// Set the update interval in milliseconds.
    /// Requires `ModifyConfig` capability.
    pub fn set_update_interval(&mut self, ms: u64) -> Result<(), PluginError> {
        self.check_capability(&PluginCapability::ModifyConfig)?;
        self.state.update_interval_ms = ms;
        Ok(())
    }

    /// Current system info (hostname, OS, kernel).
    /// Requires `ReadSystemInfo` capability.
    pub fn system_info(&self) -> crate::domain::metrics::SystemInfo {
        self.state.sys_info.clone()
    }

    /// Plugin-specific data directory (`~/.config/xtop/plugins/<plugin_id>/`).
    pub fn data_dir(&self) -> &std::path::Path {
        &self.plugin_data_dir
    }

    /// Current AppState read-only snapshot for widget rendering data.
    /// Requires `ReadSystemInfo` capability.
    pub fn state(&self) -> &AppState {
        self.state
    }
}

/// The core trait that every plugin must implement.
///
/// All methods have default empty implementations so plugins only
/// override what they need.
pub trait Plugin: Debug + Send {
    /// Static metadata about this plugin.
    fn manifest(&self) -> PluginManifest;

    /// Called once when the plugin is loaded and enabled.
    fn on_enable(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called once when the plugin is disabled or xtop shuts down.
    fn on_disable(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called on every tick (every ~1s by default).
    fn on_tick(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called when a key is pressed.
    /// Return `Ok(true)` if the plugin consumed the key event.
    fn on_key(&mut self, _ctx: &mut PluginContext, _key: &str) -> Result<bool, PluginError> {
        Ok(false)
    }

    /// Optionally provide additional system data.
    /// The returned provider is merged into the main data stream via CompositeProvider.
    fn data_provider(&self) -> Option<Box<dyn SystemDataProvider>> {
        None
    }

    /// Optionally register a custom widget for TUI rendering.
    fn widget(&self) -> Option<WidgetRegistration> {
        None
    }

    /// Execute a named command with string parameters.
    /// Used by external agents (AI, CLI, IPC) to interact with the plugin.
    ///
    /// Returns a JSON-like string response.
    fn execute(
        &mut self,
        _ctx: &mut PluginContext,
        _action: &str,
        _params: &str,
    ) -> Result<String, PluginError> {
        Err(PluginError::UnknownAction(_action.to_string()))
    }
}
