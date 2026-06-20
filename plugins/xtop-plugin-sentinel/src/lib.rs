pub mod alert;
pub mod mcp;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Debug;
use xtop_core::application::state::AppState;
use xtop_core::domain::metrics::ProcessInfo;
use xtop_core::domain::plugin::{
    Plugin, PluginCapability, PluginContext, PluginError, PluginManifest, WidgetRegistration,
};

use alert::{SentinelAlert, Severity};

// ---------------------------------------------------------------------------
// Known threat patterns (Rule 6)
// ---------------------------------------------------------------------------
const KNOWN_THREAT_NAMES: &[&str] = &[
    "minerd",
    "cpu_miner",
    "xmrig",
    "kdevtmpfsi",
    "kinsing",
    "diagree",
    "watchbog",
    "sysguard",
    "crond64",
    "mkfile",
    "sysupdate",
    "xmrig-nvidia",
    "xmrig-amd",
    "moneroocean",
];

const KNOWN_THREAT_CMDS: &[&str] = &[
    r"--donate-level",
    r"--max-cpu-usage",
    r"--threads",
    r"pool\.monero",
    r"pool\.supportxmr",
    r"mine\.monero",
];

/// Path prefixes that are suspicious for executable locations.
const SUSPICIOUS_PATH_PREFIXES: &[&str] = &[
    "/tmp/",
    "/dev/shm/",
    "/var/tmp/",
    "/proc/",
    "/private/tmp/",
    "/private/var/tmp/",
];

/// Processes allowed to be orphans (PPID=1).
const ALLOWED_ORPHANS: &[&str] = &[
    "systemd", "init", "launchd", "sshd", "login", "getty", "nginx", "apache2", "httpd", "bash",
    "sh", "zsh", "tmux", "screen",
];

/// Browsers whose children are monitored (Rule 5).
const BROWSER_NAMES: &[&str] = &[
    "chrome", "firefox", "safari", "edge", "brave", "opera", "chromium",
];

/// Browser helper/sandbox processes that are allowed children.
const BROWSER_HELPERS: &[&str] = &[
    "helper",
    "plugin_container",
    "plugin_host",
    "gpu_process",
    "renderer",
    "utility",
    "crashpad",
    "updater",
];

/// Pipe/download patterns for Rule 7.
const PIPE_PATTERNS: &[&str] = &[
    r"curl\s+.*\|\s*(ba|z)?sh",
    r"wget\s+.*\|\s*(ba|z)?sh",
    r"curl\s+.*\s*ba(?:sh)?\s*$",
    r"python3?\s+-c\s+.*(?:import|urllib|requests|socket)",
    r"base64\s+-d\s*\|",
    r"eval\s*\$\(.*curl",
    r"eval\s*\$\(.*wget",
    r"bash\s+-c\s+.*\$\(curl",
    r"bash\s+-c\s+.*\$\(wget",
];

/// High-thread-count processes that are allowed (Rule 8).
const ALLOWED_HIGH_THREAD: &[&str] = &[
    "chrome", "firefox", "code", "Code", "idea", "java", "dotnet", "python", "node", "mysqld",
    "postgres", "Xorg", "dockerd",
];

/// Maximum alerts per cycle
const MAX_ALERTS: usize = 50;

// ---------------------------------------------------------------------------
// The plugin struct
// ---------------------------------------------------------------------------

pub struct SentinelPlugin {
    enabled: bool,
    tick_count: u64,
    last_action: String,
    last_action_result: String,
    alerts: Vec<SentinelAlert>,
    // For spawn storm detection (Rule 10): name -> [(pid, start_time)]
    spawn_history: HashMap<String, Vec<(u32, u64)>>,
}

impl SentinelPlugin {
    pub fn new() -> Self {
        Self {
            enabled: true,
            tick_count: 0,
            last_action: "none".to_string(),
            last_action_result: "ok".to_string(),
            alerts: Vec::new(),
            spawn_history: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Formatting helpers
    // -----------------------------------------------------------------------

    fn fmt_process(p: &ProcessInfo) -> serde_json::Value {
        serde_json::json!({
            "pid": p.pid,
            "name": p.name,
            "cpu": (p.cpu_usage * 10.0).round() / 10.0,
            "mem_bytes": p.memory,
            "state": p.state,
            "user": p.user_id.as_deref().unwrap_or("?"),
            "cmd": p.cmd,
            "exe": p.exe_path.as_deref().unwrap_or("?"),
            "ppid": p.parent_pid,
            "threads": p.thread_count,
            "run_time": p.run_time,
            "cwd": p.cwd.as_deref().unwrap_or("?"),
        })
    }

    fn fmt_process_list(procs: &[ProcessInfo]) -> String {
        let entries: Vec<serde_json::Value> = procs.iter().map(Self::fmt_process).collect();
        serde_json::to_string(&entries).unwrap_or_default()
    }

    fn fmt_alert(a: &SentinelAlert) -> serde_json::Value {
        serde_json::json!({
            "rule": a.rule,
            "severity": format!("{:?}", a.severity),
            "pid": a.pid,
            "process": a.process_name,
            "message": a.message,
        })
    }

    // -----------------------------------------------------------------------
    // System summary
    // -----------------------------------------------------------------------

    fn system_summary(&self, ctx: &PluginContext) -> String {
        let snap = ctx.snapshot();
        let cpu_pct: f64 =
            snap.cpus.iter().map(|c| c.usage).sum::<f64>() / snap.cpus.len().max(1) as f64;
        let mem_gb = (snap.memory.used as f64 / 1073741824.0 * 10.0).round() / 10.0;
        let mem_total_gb = (snap.memory.total as f64 / 1073741824.0 * 10.0).round() / 10.0;
        let net_ifaces: Vec<&str> = snap.networks.iter().map(|n| n.name.as_str()).collect();

        serde_json::to_string(&serde_json::json!({
            "cpu_avg": (cpu_pct * 10.0).round() / 10.0,
            "mem_used_gb": mem_gb,
            "mem_total_gb": mem_total_gb,
            "mem_pct": snap.memory.percent.round() as u64,
            "processes": snap.processes.len(),
            "disks": snap.disks.len(),
            "interfaces": net_ifaces,
            "uptime_secs": snap.uptime,
            "hostname": snap.sys_info.hostname,
            "alerts": self.alerts.len(),
        }))
        .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // Search / listing
    // -----------------------------------------------------------------------

    fn search_processes(&self, ctx: &PluginContext, params: &str) -> Result<String, PluginError> {
        let (pattern_str, fields) = if let Some(idx) = params.find(",fields=") {
            let pat = &params[..idx];
            let fields_part = &params[idx + 8..];
            (pat, fields_part.split(',').collect::<Vec<&str>>())
        } else {
            (params, vec!["name"])
        };

        let pattern_str = pattern_str.trim();
        if pattern_str.is_empty() {
            return Err(PluginError::Recoverable(
                "search pattern cannot be empty".into(),
            ));
        }

        let re = Regex::new(pattern_str)
            .map_err(|e| PluginError::Recoverable(format!("invalid regex: {e}")))?;

        let snap = ctx.snapshot();
        let mut matched: Vec<ProcessInfo> = snap
            .processes
            .into_iter()
            .filter(|p| {
                fields.iter().any(|f| match *f {
                    "name" => re.is_match(&p.name),
                    "cmd" => re.is_match(&p.cmd),
                    "user" => p.user_id.as_deref().map_or(false, |u| re.is_match(u)),
                    "state" => re.is_match(&p.state),
                    "exe" => p.exe_path.as_deref().map_or(false, |e| re.is_match(e)),
                    "cwd" => p.cwd.as_deref().map_or(false, |c| re.is_match(c)),
                    _ => false,
                })
            })
            .collect();

        matched.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matched.truncate(100);
        Ok(Self::fmt_process_list(&matched))
    }

    fn top_processes(&self, ctx: &PluginContext, params: &str) -> Result<String, PluginError> {
        let (count_str, filter_pattern) = if let Some(idx) = params.find(",filter=") {
            let cnt = &params[..idx];
            let pat = &params[idx + 8..];
            (cnt, Some(pat))
        } else {
            (params, None)
        };

        let count = count_str.parse::<usize>().unwrap_or(10);
        if count == 0 {
            return Err(PluginError::Recoverable("count must be > 0".into()));
        }
        let snap = ctx.snapshot();
        let mut procs: Vec<ProcessInfo> = snap.processes;

        if let Some(pattern) = filter_pattern {
            let re = Regex::new(pattern)
                .map_err(|e| PluginError::Recoverable(format!("invalid regex in filter: {e}")))?;
            procs.retain(|p| {
                re.is_match(&p.name)
                    || re.is_match(&p.cmd)
                    || p.exe_path.as_deref().map_or(false, |e| re.is_match(e))
            });
        }

        procs.truncate(count);
        Ok(Self::fmt_process_list(&procs))
    }

    fn process_info(&self, ctx: &PluginContext, pid_str: &str) -> Result<String, PluginError> {
        let pid = pid_str
            .parse::<u32>()
            .map_err(|_| PluginError::Recoverable(format!("invalid pid: {pid_str}")))?;
        let snap = ctx.snapshot();
        let proc = snap
            .processes
            .iter()
            .find(|p| p.pid == pid)
            .ok_or_else(|| PluginError::Recoverable(format!("process {pid} not found")))?;
        Ok(serde_json::to_string(&Self::fmt_process(proc)).unwrap_or_default())
    }

    // -----------------------------------------------------------------------
    // Heuristic rules
    // -----------------------------------------------------------------------

    /// Rule 1: Executable running from a suspicious path.
    fn rule_suspicious_exe_path(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        let exe = proc.exe_path.as_deref()?;
        if SUSPICIOUS_PATH_PREFIXES.iter().any(|p| exe.starts_with(p)) {
            Some(SentinelAlert::new(
                "suspicious_exe_path",
                Severity::Critical,
                proc.pid,
                proc.name.clone(),
                format!("executable runs from suspicious path: {exe}"),
            ))
        } else {
            None
        }
    }

    /// Rule 2: Orphan process (PPID=1) that is not a known system daemon.
    fn rule_orphan_process(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        if proc.parent_pid != Some(1) {
            return None;
        }
        let name_lower = proc.name.to_lowercase();
        if ALLOWED_ORPHANS.iter().any(|a| name_lower.contains(a)) {
            return None;
        }
        let severity = if proc.run_time < 60 {
            Severity::Critical
        } else {
            Severity::Warning
        };
        Some(SentinelAlert::new(
            "orphan_process",
            severity,
            proc.pid,
            proc.name.clone(),
            format!(
                "orphan process (PPID=1), running for {}s, exe: {}",
                proc.run_time,
                proc.exe_path.as_deref().unwrap_or("?"),
            ),
        ))
    }

    /// Rule 3: Process masquerading (name != exe file stem OR system name not at canonical path).
    fn rule_masquerading(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        let exe = match proc.exe_path.as_deref() {
            Some(e) => e,
            None => return None,
        };
        // Extract file stem from exe
        let stem = std::path::Path::new(exe)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let name_lower = proc.name.to_lowercase();
        let stem_lower = stem.to_lowercase();

        // Check if name is a known system process but exe is not at a canonical path
        let known_system_names = ["svchost", "lsass", "launchd", "sshd", "systemd", "init"];
        if known_system_names.contains(&name_lower.as_str()) {
            let canonical = exe.starts_with("/usr/")
                || exe.starts_with("/bin/")
                || exe.starts_with("/sbin/")
                || exe.starts_with("/System/");
            if !canonical {
                return Some(SentinelAlert::new(
                    "process_masquerading",
                    Severity::Critical,
                    proc.pid,
                    proc.name.clone(),
                    format!(
                        "process name '{}' masquerades as system process; exe: {exe}",
                        proc.name
                    ),
                ));
            }
        }

        // Check if name differs significantly from exe file stem
        if name_lower != stem_lower && !stem_lower.is_empty() {
            // Allow common cases like "python3" -> exe "/usr/bin/python3.11"
            if !exe.contains(&name_lower) && !name_lower.contains(&stem_lower) {
                return Some(SentinelAlert::new(
                    "process_masquerading",
                    Severity::Warning,
                    proc.pid,
                    proc.name.clone(),
                    format!(
                        "name '{}' differs from exe stem '{stem}' ({exe})",
                        proc.name
                    ),
                ));
            }
        }

        None
    }

    /// Rule 4: Privilege escalation (EUID != UID).
    fn rule_privilege_escalation(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        let euid = match proc.effective_user_id.as_deref() {
            Some(u) => u,
            None => return None,
        };
        let uid = match proc.user_id.as_deref() {
            Some(u) => u,
            None => return None,
        };
        if euid == uid {
            return None;
        }
        // Known SUID binaries that are allowed
        let known_suid = [
            "/usr/bin/sudo",
            "/usr/bin/passwd",
            "/bin/ping",
            "/usr/bin/ping",
            "/bin/su",
            "/usr/bin/su",
            "/usr/bin/newgrp",
            "/usr/bin/gpasswd",
            "/usr/bin/chsh",
            "/usr/bin/chfn",
            "/usr/bin/mount",
            "/usr/bin/umount",
        ];
        let exe = proc.exe_path.as_deref().unwrap_or("");
        if known_suid.contains(&exe) {
            return None;
        }
        let severity = if euid == "0" {
            Severity::Critical
        } else {
            Severity::Warning
        };
        Some(SentinelAlert::new(
            "privilege_escalation",
            severity,
            proc.pid,
            proc.name.clone(),
            format!("EUID ({euid}) != UID ({uid}), exe: {exe}"),
        ))
    }

    /// Rule 5: Suspicious child of a browser process.
    fn rule_suspicious_child_of_browser(
        &self,
        proc: &ProcessInfo,
        parent_map: &HashMap<u32, &ProcessInfo>,
    ) -> Option<SentinelAlert> {
        let ppid = match proc.parent_pid {
            Some(pid) => pid,
            None => return None,
        };
        let parent = match parent_map.get(&ppid) {
            Some(p) => p,
            None => return None,
        };
        let parent_lower = parent.name.to_lowercase();
        let is_browser = BROWSER_NAMES.iter().any(|b| parent_lower.contains(b));
        if !is_browser {
            return None;
        }
        let child_lower = proc.name.to_lowercase();
        let is_helper = BROWSER_HELPERS.iter().any(|h| child_lower.contains(h));
        if is_helper {
            return None;
        }
        Some(SentinelAlert::new(
            "suspicious_child_of_browser",
            Severity::Warning,
            proc.pid,
            proc.name.clone(),
            format!(
                "browser '{}' spawned unknown child '{}'",
                parent.name, proc.name
            ),
        ))
    }

    /// Rule 6: Known threat pattern (name or cmd matches miner/rootkit names).
    fn rule_known_threat_pattern(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        let name_lower = proc.name.to_lowercase();
        if KNOWN_THREAT_NAMES.iter().any(|t| name_lower.contains(t)) {
            return Some(SentinelAlert::new(
                "known_threat_pattern",
                Severity::Critical,
                proc.pid,
                proc.name.clone(),
                format!("process name matches known threat pattern: {}", proc.name),
            ));
        }
        let cmd_joined = proc.cmd_full.join(" ").to_lowercase();
        if KNOWN_THREAT_CMDS.iter().any(|t| {
            Regex::new(t)
                .ok()
                .map_or(false, |re| re.is_match(&cmd_joined))
        }) {
            return Some(SentinelAlert::new(
                "known_threat_pattern",
                Severity::Critical,
                proc.pid,
                proc.name.clone(),
                format!("command line matches known threat pattern: {}", proc.cmd),
            ));
        }
        None
    }

    /// Rule 7: Suspicious pipe/download pattern in command line.
    fn rule_suspicious_pipe_or_download(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        let cmd_joined = proc.cmd_full.join(" ");
        for pattern in PIPE_PATTERNS {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(&cmd_joined) {
                    return Some(SentinelAlert::new(
                        "suspicious_pipe_or_download",
                        Severity::Critical,
                        proc.pid,
                        proc.name.clone(),
                        format!("command matches pipe/download pattern: {}", proc.cmd),
                    ));
                }
            }
        }
        None
    }

    /// Rule 8: High thread count anomaly.
    fn rule_high_thread_anomaly(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        if proc.thread_count < 500 {
            return None;
        }
        let name_lower = proc.name.to_lowercase();
        if ALLOWED_HIGH_THREAD.iter().any(|a| name_lower.contains(a)) {
            return None;
        }
        let severity = if proc.thread_count > 1000 || proc.cpu_usage > 200.0 {
            Severity::Critical
        } else {
            Severity::Warning
        };
        Some(SentinelAlert::new(
            "high_thread_anomaly",
            severity,
            proc.pid,
            proc.name.clone(),
            format!(
                "{} threads (CPU: {:.1}%)",
                proc.thread_count, proc.cpu_usage
            ),
        ))
    }

    /// Rule 9: Suspicious file descriptor anomaly.
    fn rule_suspicious_fd_anomaly(&self, proc: &ProcessInfo) -> Option<SentinelAlert> {
        if proc.open_files < 1000 {
            return None;
        }
        let name_lower = proc.name.to_lowercase();
        let allowed = [
            "mysql", "postgres", "nginx", "httpd", "apache", "chrome", "firefox", "code", "java",
            "dotnet", "dockerd",
        ];
        if allowed.iter().any(|a| name_lower.contains(a)) {
            return None;
        }
        Some(SentinelAlert::new(
            "suspicious_fd_anomaly",
            Severity::Info,
            proc.pid,
            proc.name.clone(),
            format!("{} open file descriptors", proc.open_files),
        ))
    }

    /// Rule 10: Spawn storm detection.
    fn rule_spawn_storm(&mut self, proc: &ProcessInfo, now_run_time: u64) -> Option<SentinelAlert> {
        if proc.run_time > 120 {
            return None;
        }
        let entry = self.spawn_history.entry(proc.name.clone()).or_default();
        entry.push((proc.pid, proc.start_time));
        // Purge entries older than 120s
        entry.retain(|(_, start)| now_run_time.saturating_sub(*start) < 120);
        if entry.len() > 5 {
            Some(SentinelAlert::new(
                "recent_spawn_storm",
                Severity::Warning,
                proc.pid,
                proc.name.clone(),
                format!("{} new instances in last 120s", entry.len()),
            ))
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------
    // Main analyzer
    // -----------------------------------------------------------------------

    fn analyze_processes(&mut self, ctx: &PluginContext) -> Vec<SentinelAlert> {
        let snap = ctx.snapshot();
        let mut alerts: Vec<SentinelAlert> = Vec::new();

        // Build parent PID map for Rule 5
        let parent_map: HashMap<u32, &ProcessInfo> =
            snap.processes.iter().map(|p| (p.pid, p)).collect();

        let now_run_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for proc in &snap.processes {
            // Run rules in priority order
            if let Some(a) = self.rule_suspicious_exe_path(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_masquerading(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_known_threat_pattern(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_suspicious_pipe_or_download(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_orphan_process(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_privilege_escalation(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_suspicious_child_of_browser(proc, &parent_map) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_high_thread_anomaly(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_suspicious_fd_anomaly(proc) {
                alerts.push(a);
            }
            if let Some(a) = self.rule_spawn_storm(proc, now_run_time) {
                alerts.push(a);
            }
        }

        alerts.truncate(MAX_ALERTS);
        alerts
    }

    fn parse_thresholds(params: &str) -> Result<(f64, f64, f64), PluginError> {
        let parts: Vec<&str> = params.split(',').collect();
        if parts.len() != 3 {
            return Err(PluginError::Recoverable(
                "expected cpu,mem,disk (3 comma-separated values)".into(),
            ));
        }
        let cpu = parts[0]
            .parse::<f64>()
            .map_err(|e| PluginError::Recoverable(format!("invalid cpu threshold: {e}")))?;
        let mem = parts[1]
            .parse::<f64>()
            .map_err(|e| PluginError::Recoverable(format!("invalid mem threshold: {e}")))?;
        let disk = parts[2]
            .parse::<f64>()
            .map_err(|e| PluginError::Recoverable(format!("invalid disk threshold: {e}")))?;
        Ok((cpu, mem, disk))
    }
}

impl Debug for SentinelPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SentinelPlugin")
            .field("enabled", &self.enabled)
            .field("tick_count", &self.tick_count)
            .field("alerts", &self.alerts.len())
            .finish()
    }
}

impl Plugin for SentinelPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "sentinel".to_string(),
            name: "Sentinel".to_string(),
            version: "0.1.0".to_string(),
            description: "AI-aware system monitoring, management, and heuristic threat detection"
                .to_string(),
            capabilities: vec![
                PluginCapability::ReadSystemInfo,
                PluginCapability::KillProcesses,
                PluginCapability::ModifyConfig,
                PluginCapability::RenderWidgets,
            ],
        }
    }

    fn on_enable(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn on_disable(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
        self.enabled = false;
        Ok(())
    }

    fn on_tick(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        self.tick_count += 1;
        if self.tick_count % 5 == 0 {
            self.alerts = self.analyze_processes(ctx);
        }
        Ok(())
    }

    fn widget(&self) -> Option<WidgetRegistration> {
        Some(WidgetRegistration {
            name: "sentinel".to_string(),
            render: std::sync::Arc::new(|f: &mut ratatui::Frame, _state: &AppState, area: Rect| {
                use xtop_core::domain::theme::hex_to_rgb_pub;
                let bg = hex_to_rgb_pub("#1a1b2e");
                let fg = hex_to_rgb_pub("#7ec8e3");
                let accent = hex_to_rgb_pub("#c084fc");

                let block = Block::default()
                    .title(" Sentinel ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(accent[0], accent[1], accent[2])))
                    .style(
                        Style::default()
                            .bg(Color::Rgb(bg[0], bg[1], bg[2]))
                            .fg(Color::Rgb(fg[0], fg[1], fg[2])),
                    );
                let inner = block.inner(area);
                f.render_widget(block, area);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(inner);

                let status = Paragraph::new("Agent: monitoring for threats -- use MCP to interact")
                    .style(Style::default().fg(Color::Rgb(accent[0], accent[1], accent[2])));
                f.render_widget(status, chunks[0]);

                let info = Paragraph::new("run xtop mcp for AI tool integration")
                    .style(Style::default().fg(Color::Rgb(fg[0], fg[1], fg[2])));
                f.render_widget(info, chunks[1]);
            }),
        })
    }

    fn execute(
        &mut self,
        ctx: &mut PluginContext,
        action: &str,
        params: &str,
    ) -> Result<String, PluginError> {
        self.last_action = format!("{}({})", action, params);

        let result = match action {
            "system.summary" => Ok(self.system_summary(ctx)),
            "processes.top" => self.top_processes(ctx, params),
            "processes.search" => self.search_processes(ctx, params),
            "process.info" => self.process_info(ctx, params),
            "process.kill" => {
                let pid = params
                    .parse::<u32>()
                    .map_err(|_| PluginError::Recoverable(format!("invalid pid: {params}")))?;
                let ok = ctx
                    .kill_process(pid)
                    .map_err(|e| PluginError::Recoverable(e.to_string()))?;
                Ok(serde_json::to_string(&serde_json::json!({
                    "killed": ok,
                    "pid": pid,
                }))
                .unwrap_or_default())
            }
            "process.alerts" => {
                let alerts_json: Vec<serde_json::Value> =
                    self.alerts.iter().map(Self::fmt_alert).collect();
                Ok(serde_json::to_string(&alerts_json).unwrap_or_default())
            }
            "threshold.set" => {
                let (cpu, mem, disk) = Self::parse_thresholds(params)?;
                ctx.set_alert_thresholds(cpu, mem, disk)
                    .map_err(|e| PluginError::Recoverable(e.to_string()))?;
                Ok(serde_json::to_string(&serde_json::json!({
                    "cpu": cpu, "mem": mem, "disk": disk, "set": true,
                }))
                .unwrap_or_default())
            }
            "threshold.get" => {
                let alerts = ctx.state().alerts;
                Ok(serde_json::to_string(&serde_json::json!({
                    "cpu": alerts.cpu_high,
                    "mem": alerts.mem_high,
                    "disk": alerts.disk_high,
                }))
                .unwrap_or_default())
            }
            "config.get" => {
                let s = ctx.state();
                Ok(serde_json::to_string(&serde_json::json!({
                    "theme": s.current_theme.name,
                    "layout": s.current_layout_name(),
                    "interval_ms": s.update_interval_ms,
                    "hostname": s.sys_info.hostname,
                }))
                .unwrap_or_default())
            }
            "config.set" => {
                if let Some(val) = params.strip_prefix("interval_ms=") {
                    let ms = val.parse::<u64>().map_err(|e| {
                        PluginError::Recoverable(format!("invalid interval_ms: {e}"))
                    })?;
                    ctx.set_update_interval(ms)
                        .map_err(|e| PluginError::Recoverable(e.to_string()))?;
                    Ok(serde_json::to_string(&serde_json::json!({
                        "interval_ms": ms, "set": true,
                    }))
                    .unwrap_or_default())
                } else if let Some(name) = params.strip_prefix("theme=") {
                    let ok = ctx
                        .set_theme_by_name(name)
                        .map_err(|e| PluginError::Recoverable(e.to_string()))?;
                    Ok(serde_json::to_string(&serde_json::json!({
                        "theme": name, "set": ok,
                    }))
                    .unwrap_or_default())
                } else if let Some(name) = params.strip_prefix("layout=") {
                    let ok = ctx
                        .set_layout_by_name(name)
                        .map_err(|e| PluginError::Recoverable(e.to_string()))?;
                    Ok(serde_json::to_string(&serde_json::json!({
                        "layout": name, "set": ok,
                    }))
                    .unwrap_or_default())
                } else {
                    Err(PluginError::Recoverable(
                        "expected interval_ms=<ms>, theme=<name>, or layout=<name>".into(),
                    ))
                }
            }
            "alerts.status" => {
                let critical = self
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, Severity::Critical))
                    .count();
                let warning = self
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, Severity::Warning))
                    .count();
                let info_count = self
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, Severity::Info))
                    .count();
                let top: Vec<serde_json::Value> =
                    self.alerts.iter().take(5).map(Self::fmt_alert).collect();
                Ok(serde_json::to_string(&serde_json::json!({
                    "total": self.alerts.len(),
                    "critical": critical,
                    "warning": warning,
                    "info": info_count,
                    "alerts": top,
                }))
                .unwrap_or_default())
            }
            "plugin.status" => {
                let critical = self
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, Severity::Critical))
                    .count();
                Ok(serde_json::to_string(&serde_json::json!({
                    "enabled": self.enabled,
                    "ticks": self.tick_count,
                    "last_action": self.last_action,
                    "last_result": self.last_action_result,
                    "active_alerts": self.alerts.len(),
                    "critical_alerts": critical,
                }))
                .unwrap_or_default())
            }
            _ => {
                return Err(PluginError::UnknownAction(action.to_string()));
            }
        };

        match &result {
            Ok(r) => self.last_action_result = format!("ok ({} chars)", r.len()),
            Err(e) => self.last_action_result = format!("error: {e}"),
        }

        result
    }
}
