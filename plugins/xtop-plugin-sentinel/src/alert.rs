use serde::Serialize;

/// Severity level for a Sentinel alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    /// Low-priority, informative
    Info,
    /// Moderate concern, shown in widget
    Warning,
    /// High confidence threat
    Critical,
}

/// A single security alert produced by a heuristic rule.
#[derive(Debug, Clone, Serialize)]
pub struct SentinelAlert {
    /// Short rule name (e.g. "suspicious_exe_path", "orphan_process")
    pub rule: &'static str,
    /// Severity level
    pub severity: Severity,
    /// Process ID that triggered the alert
    pub pid: u32,
    /// Process name
    pub process_name: String,
    /// Human-readable detail message
    pub message: String,
}

impl SentinelAlert {
    pub fn new(
        rule: &'static str,
        severity: Severity,
        pid: u32,
        process_name: String,
        message: String,
    ) -> Self {
        Self {
            rule,
            severity,
            pid,
            process_name,
            message,
        }
    }
}
