//! Unit tests for the kernel app state (kept in their own file so
//! `state/app.rs` stays under the 600-line structural-audit cap).
//!
//! Covers layout cycling over the Detail preset extras and the process-sort
//! direction state machine + the ordering it produces.

use crate::config::default_alerts;
use crate::config::Config;
use crate::state::app::AppState;
use crate::state::ProcessSortBy;
use crate::theme::Theme;
use xtop_layout::{default_layouts, LayoutDef, LayoutMode};
use xtop_plugin_api::model::{
    CpuInfo, DiskIOInfo, DiskInfo, LoadAvg, MemoryInfo, NetworkInfo, ProcessInfo, SwapInfo,
    SystemInfo, SystemSnapshot,
};

fn test_state(defs: Vec<LayoutDef>) -> AppState {
    let theme = Theme {
        name: "test".into(),
        background: [0, 0, 0],
        foreground: [255, 255, 255],
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
    use crate::state::FullScreenWidget;
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

#[test]
fn test_layout_cycle_reaches_preset_extras() {
    // DR-UX6: ten embedded defaults — the seven mode-bound layouts first
    // (slots 0-6), then the Detail preset extras (slots 7-9). The layout
    // cycling key walks every definition in file order and wraps.
    let defs = default_layouts();
    assert_eq!(defs.len(), 10, "7 mode-bound defaults + 3 detail presets");
    let mut state = test_state(defs);
    assert_eq!(state.current_layout_name(), "Dashboard");

    for name in [
        "Vertical",
        "Horizontal",
        "CPU Focus",
        "Memory Focus",
        "Network Focus",
        "Process Focus",
    ] {
        state.next_layout();
        assert_eq!(state.current_layout_name(), name);
    }
    // After the modes the key reaches the extras by name...
    state.next_layout();
    assert_eq!(state.current_layout_name(), "Detail Dashboard");
    state.next_layout();
    assert_eq!(state.current_layout_name(), "Detail Network");
    state.next_layout();
    assert_eq!(state.current_layout_name(), "Detail Processes");
    // ...and wraps back to the first mode.
    state.next_layout();
    assert_eq!(state.current_layout_name(), "Dashboard");
}

#[test]
fn test_sort_cycle_covers_columns_and_directions() {
    let mut state = test_state(vec![]);
    // Boot: CPU%, descending (classic high-first order).
    assert_eq!(state.process_sort, ProcessSortBy::Cpu);
    assert!(state.process_sort_desc);

    // Each press alternates: flip the current column's direction, then
    // advance to the next column (which starts descending). The full
    // cycle visits every (column, direction) pair and wraps to the boot
    // state after 4 columns x 2 directions presses.
    let expected: &[(ProcessSortBy, bool)] = &[
        (ProcessSortBy::Cpu, false),    // press 1: toggle desc -> asc
        (ProcessSortBy::Memory, true),  // press 2: next column, desc
        (ProcessSortBy::Memory, false), // press 3: toggle
        (ProcessSortBy::Pid, true),     // press 4: next column, desc
        (ProcessSortBy::Pid, false),    // press 5: toggle
        (ProcessSortBy::Name, true),    // press 6: next column, desc
        (ProcessSortBy::Name, false),   // press 7: toggle
        (ProcessSortBy::Cpu, true),     // press 8: wrap to CPU, desc
    ];
    for &(column, desc) in expected {
        state.cycle_sort();
        assert_eq!(state.process_sort, column);
        assert_eq!(state.process_sort_desc, desc);
    }
}

#[test]
fn test_sorted_processes_honors_direction() {
    let snap = snapshot_with(vec![
        proc(101, "alpha", 10.0, 100),
        proc(102, "bravo", 90.0, 300),
        proc(103, "charlie", 50.0, 200),
    ]);

    // Descending (default): largest CPU% first.
    let state = test_state(vec![]);
    assert!(state.process_sort_desc);
    let view = state.sorted_processes(&snap);
    let pids: Vec<u32> = view.iter().map(|p| p.pid).collect();
    assert_eq!(pids, [102, 103, 101]);

    // Ascending: smallest CPU% first.
    let mut state = test_state(vec![]);
    state.process_sort_desc = false;
    let view = state.sorted_processes(&snap);
    let pids: Vec<u32> = view.iter().map(|p| p.pid).collect();
    assert_eq!(pids, [101, 103, 102]);

    // PID column: descending (high pid first) in its default direction,
    // ascending (low pid first) when toggled.
    let mut state = test_state(vec![]);
    state.process_sort = ProcessSortBy::Pid;
    let view = state.sorted_processes(&snap);
    let pids: Vec<u32> = view.iter().map(|p| p.pid).collect();
    assert_eq!(pids, [103, 102, 101]);
    state.process_sort_desc = false;
    let view = state.sorted_processes(&snap);
    let pids: Vec<u32> = view.iter().map(|p| p.pid).collect();
    assert_eq!(pids, [101, 102, 103]);
}

/// Minimal `ProcessInfo` for sorting tests.
fn proc(pid: u32, name: &str, cpu_usage: f64, memory: u64) -> ProcessInfo {
    ProcessInfo {
        pid,
        name: name.to_string(),
        cpu_usage,
        memory,
        user_id: None,
        state: "running".into(),
        cmd: name.to_string(),
        exe_path: None,
        parent_pid: None,
        cmd_full: Vec::new(),
        start_time: 0,
        run_time: 0,
        effective_user_id: None,
        group_id: None,
        cwd: None,
        thread_count: 1,
        open_files: 0,
        open_files_limit: 0,
        disk_total_read_bytes: 0,
        disk_total_write_bytes: 0,
        environ: Vec::new(),
        session_id: None,
    }
}

/// A `SystemSnapshot` carrying exactly the given processes.
fn snapshot_with(processes: Vec<ProcessInfo>) -> SystemSnapshot {
    SystemSnapshot {
        cpus: vec![CpuInfo {
            name: "cpu0".into(),
            usage: 0.0,
            cpu_id: 0,
            frequency: 0,
            governor: String::new(),
            temp_c: None,
        }],
        memory: MemoryInfo {
            total: 0,
            used: 0,
            available: 0,
            free: 0,
            percent: 0.0,
        },
        swap: SwapInfo {
            total: 0,
            used: 0,
            free: 0,
            percent: 0.0,
        },
        disks: vec![DiskInfo {
            mount_point: "/".into(),
            total_space: 0,
            available_space: 0,
            used_space: 0,
            percent: 0.0,
            file_system: String::new(),
            mount_options: String::new(),
        }],
        networks: vec![NetworkInfo {
            name: "lo".into(),
            received: 0,
            transmitted: 0,
            rx_speed: 0.0,
            tx_speed: 0.0,
            ip: Vec::new(),
        }],
        processes,
        load_avg: LoadAvg {
            one: 0.0,
            five: 0.0,
            fifteen: 0.0,
        },
        uptime: 0,
        cpu_temp: 0.0,
        disk_io: vec![DiskIOInfo {
            name: "sda".into(),
            read_bytes: 0,
            write_bytes: 0,
            read_speed: 0.0,
            write_speed: 0.0,
        }],
        batteries: Vec::new(),
        gpus: Vec::new(),
        sys_info: SystemInfo::default(),
    }
}
