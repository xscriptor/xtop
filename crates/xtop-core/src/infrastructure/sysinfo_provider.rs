use crate::domain::metrics::*;
use crate::domain::system_info::SystemDataProvider;
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind,
    RefreshKind, Signal, System,
};

pub const DEFAULT_MAX_PROCESSES: usize = 200;

pub struct SysinfoProvider {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    prev_disk_read: HashMap<String, u64>,
    prev_disk_write: HashMap<String, u64>,
    prev_net_rx: HashMap<String, u64>,
    prev_net_tx: HashMap<String, u64>,
    last_refresh: Instant,
    cached_sys_info: SystemInfo,
    max_processes: usize,
}

impl Default for SysinfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SysinfoProvider {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        let info = SystemInfo {
            hostname: System::host_name().unwrap_or_default(),
            os_version: System::long_os_version().unwrap_or_default(),
            kernel: System::kernel_version().unwrap_or_default(),
            desktop_env: std::env::var("XDG_CURRENT_DESKTOP")
                .or_else(|_| std::env::var("DESKTOP_SESSION"))
                .unwrap_or_default(),
            shell: std::env::var("SHELL")
                .or_else(|_| std::env::var("ComSpec"))
                .unwrap_or_default(),
        };
        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            prev_disk_read: HashMap::new(),
            prev_disk_write: HashMap::new(),
            prev_net_rx: HashMap::new(),
            prev_net_tx: HashMap::new(),
            last_refresh: Instant::now(),
            cached_sys_info: info,
            max_processes: DEFAULT_MAX_PROCESSES,
        }
    }
}

impl SystemDataProvider for SysinfoProvider {
    fn refresh_all(&mut self) {
        self.sys.refresh_all();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);
    }

    fn snapshot(&self) -> SystemSnapshot {
        let cpus: Vec<CpuInfo> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, c)| CpuInfo {
                name: c.name().to_string(),
                usage: c.cpu_usage() as f64,
                cpu_id: i,
                frequency: c.frequency(),
                governor: read_cpu_governor(i),
            })
            .collect();

        let total_mem = self.sys.total_memory();
        let used_mem = self.sys.used_memory();
        let memory = MemoryInfo {
            total: total_mem,
            used: used_mem,
            available: self.sys.available_memory(),
            free: self.sys.free_memory(),
            percent: if total_mem > 0 {
                (used_mem as f64 / total_mem as f64) * 100.0
            } else {
                0.0
            },
        };

        let total_swap = self.sys.total_swap();
        let used_swap = self.sys.used_swap();
        let swap = SwapInfo {
            total: total_swap,
            used: used_swap,
            free: self.sys.free_swap(),
            percent: if total_swap > 0 {
                (used_swap as f64 / total_swap as f64) * 100.0
            } else {
                0.0
            },
        };

        let mount_info = read_mount_options();

        let disks: Vec<DiskInfo> = self
            .disks
            .iter()
            .map(|d| {
                let mp = d.mount_point().to_string_lossy().to_string();
                let total = d.total_space();
                let available = d.available_space();
                let used = total - available;
                let opts = mount_info.get(&mp).cloned().unwrap_or_default();
                DiskInfo {
                    mount_point: mp,
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    percent: if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                    file_system: d.file_system().to_string_lossy().to_string(),
                    mount_options: opts,
                }
            })
            .collect();

        let iface_ips = read_interface_ips();

        let networks: Vec<NetworkInfo> = self
            .networks
            .iter()
            .map(|(name, n)| {
                let rx = n.received();
                let tx = n.transmitted();
                let rx_speed = if let Some(prev) = self.prev_net_rx.get(name) {
                    let elapsed = self.last_refresh.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        (rx.saturating_sub(*prev)) as f64 / elapsed
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let tx_speed = if let Some(prev) = self.prev_net_tx.get(name) {
                    let elapsed = self.last_refresh.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        (tx.saturating_sub(*prev)) as f64 / elapsed
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                NetworkInfo {
                    name: name.clone(),
                    received: rx,
                    transmitted: tx,
                    rx_speed,
                    tx_speed,
                    ip: iface_ips.get(name).cloned().unwrap_or_default(),
                }
            })
            .collect();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut procs: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let start = p.start_time();
                let run = if start > 0 { now.saturating_sub(start) } else { 0 };
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().to_string(),
                    cpu_usage: p.cpu_usage() as f64,
                    memory: p.memory(),
                    user_id: p.user_id().map(|u| u.to_string()),
                    state: format!("{:?}", p.status()),
                    cmd: p.cmd().first().map(|c| c.to_string_lossy().to_string()).unwrap_or_default(),

                    // P0
                    exe_path: p.exe().map(|e| e.to_string_lossy().to_string()),
                    parent_pid: p.parent().map(|ppid| ppid.as_u32()),
                    cmd_full: p.cmd().iter().map(|c| c.to_string_lossy().to_string()).collect(),

                    // P1
                    start_time: start,
                    run_time: run,
                    effective_user_id: p.effective_user_id().map(|u| u.to_string()),
                    group_id: p.group_id().map(|g| g.to_string()),
                    cwd: p.cwd().map(|c| c.to_string_lossy().to_string()),
                    thread_count: read_thread_count(p.pid()),

                    // P2
                    open_files: p.open_files().unwrap_or(0) as u64,
                    open_files_limit: p.open_files_limit().unwrap_or(0) as u64,
                    disk_total_read_bytes: p.disk_usage().total_read_bytes,
                    disk_total_write_bytes: p.disk_usage().total_written_bytes,
                    environ: p.environ().iter().map(|e| e.to_string_lossy().to_string()).collect(),
                    session_id: p.session_id().map(|s| s.as_u32()),
                }
            })
            .collect();
        procs.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        procs.truncate(self.max_processes);

        let mut max_temp = 0.0f32;
        for component in &self.components {
            if let Some(temp) = component.temperature() {
                if temp > max_temp {
                    max_temp = temp;
                }
            }
        }

        let load = System::load_average();

        SystemSnapshot {
            cpu_temp: max_temp as f64,
            cpus,
            memory,
            swap,
            disks,
            networks,
            processes: procs,
            load_avg: LoadAvg {
                one: load.one,
                five: load.five,
                fifteen: load.fifteen,
            },
            uptime: System::uptime(),
            disk_io: self.disk_io_inner(),
            batteries: read_batteries(),
            gpus: read_gpu_info(),
            dockers: vec![],
            sys_info: SystemInfo::default(),
        }
    }

    fn disk_io(&self) -> Vec<DiskIOInfo> {
        self.disk_io_inner()
    }

    fn system_info(&self) -> SystemInfo {
        self.cached_sys_info.clone()
    }

    fn kill_process(&self, pid: u32) -> bool {
        if let Some(process) = self.sys.process(Pid::from(pid as usize)) {
            // Only allow killing processes owned by the same user (safety check)
            let current_uid = self
                .sys
                .process(sysinfo::get_current_pid().unwrap_or(Pid::from(0)))
                .and_then(|p| p.user_id());
            let target_uid = process.user_id();
            match (current_uid, target_uid) {
                (Some(current), Some(target)) if current == target => {
                    process.kill_with(Signal::Term).unwrap_or(false)
                }
                (Some(_), Some(_)) => false,
                _ => process.kill_with(Signal::Term).unwrap_or(false),
            }
        } else {
            false
        }
    }

    fn batteries(&self) -> Vec<BatteryInfo> {
        read_batteries()
    }

    fn gpu_info(&self) -> Vec<GpuInfo> {
        read_gpu_info()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl SysinfoProvider {
    fn disk_io_inner(&self) -> Vec<DiskIOInfo> {
        self.disks
            .iter()
            .map(|d| {
                let name = d.mount_point().to_string_lossy().to_string();
                let usage = d.usage();
                let read_bytes = usage.read_bytes;
                let write_bytes = usage.written_bytes;
                let (read_speed, write_speed) = if let (Some(prev_r), Some(prev_w)) = (
                    self.prev_disk_read.get(&name),
                    self.prev_disk_write.get(&name),
                ) {
                    let elapsed = self.last_refresh.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        (
                            (read_bytes.saturating_sub(*prev_r)) as f64 / elapsed,
                            (write_bytes.saturating_sub(*prev_w)) as f64 / elapsed,
                        )
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };
                DiskIOInfo {
                    name,
                    read_bytes,
                    write_bytes,
                    read_speed,
                    write_speed,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Platform-specific helpers with graceful fallbacks
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn read_cpu_governor(_cpu_id: usize) -> String {
    std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{_cpu_id}/cpufreq/scaling_governor"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

#[cfg(not(target_os = "linux"))]
fn read_cpu_governor(_cpu_id: usize) -> String {
    String::new()
}

#[cfg(target_os = "linux")]
fn read_mount_options() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = std::fs::read_to_string("/proc/self/mountinfo") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Format: id parent_id maj:min root mount_point options ...
            if parts.len() >= 6 {
                let mount_point = parts[4].to_string();
                let opts = parts[5].to_string();
                map.insert(mount_point, opts);
            }
        }
    }
    map
}

#[cfg(not(target_os = "linux"))]
fn read_mount_options() -> HashMap<String, String> {
    HashMap::new()
}

#[cfg(target_os = "linux")]
fn read_interface_ips() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    // Parse /proc/net/if_inet6 for IPv6 addresses
    if let Ok(content) = std::fs::read_to_string("/proc/net/if_inet6") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let addr_hex = parts[0];
                let iface = parts[4].to_string();
                if addr_hex.len() == 32 {
                    let ip: String = (0..8)
                        .map(|i| {
                            let start = i * 4;
                            let group = &addr_hex[start..start + 4];
                            let trimmed = group.trim_start_matches('0');
                            let val = u16::from_str_radix(if trimmed.is_empty() { "0" } else { trimmed }, 16).unwrap_or(0);
                            format!("{:x}", val)
                        })
                        .collect::<Vec<_>>()
                        .join(":");
                    map.entry(iface).or_default().push(ip);
                }
            }
        }
    }
    // Parse /proc/net/fib_trie for IPv4 (fallback)
    map
}

#[cfg(not(target_os = "linux"))]
fn read_interface_ips() -> HashMap<String, Vec<String>> {
    HashMap::new()
}

#[cfg(target_os = "linux")]
fn read_batteries() -> Vec<BatteryInfo> {
    let mut batteries = Vec::new();
    let power_supply = std::path::Path::new("/sys/class/power_supply");
    if !power_supply.exists() {
        return batteries;
    }
    if let Ok(entries) = std::fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let name = match entry.file_name().to_str() {
                Some(n) if n.starts_with("BAT") => n.to_string(),
                _ => continue,
            };
            let base = entry.path();
            let capacity = std::fs::read_to_string(base.join("capacity"))
                .ok()
                .and_then(|s| s.trim().parse::<f32>().ok())
                .unwrap_or(0.0);
            let state = std::fs::read_to_string(base.join("status"))
                .ok()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let charge_full = std::fs::read_to_string(base.join("charge_full"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            let charge_now = std::fs::read_to_string(base.join("charge_now"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            let cycles = std::fs::read_to_string(base.join("cycle_count"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());

            // time至full/empty estimation from power
            let power_now = std::fs::read_to_string(base.join("power_now"))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);
            let charge_full_design = std::fs::read_to_string(base.join("charge_full_design"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(1);

            let health = if charge_full_design > 0 {
                (charge_full.unwrap_or(1) as f32 / charge_full_design as f32) * 100.0
            } else {
                100.0
            };

            let (time_to_full, time_to_empty) = if power_now != 0 && power_now.abs() > 0 {
                if state == "Charging" {
                    let remaining = charge_full.unwrap_or(0).saturating_sub(charge_now.unwrap_or(0));
                    let secs = (remaining as f64 / power_now.abs() as f64 * 3600.0) as u64;
                    (Some(secs), None)
                } else if state == "Discharging" {
                    let secs = (charge_now.unwrap_or(0) as f64 / power_now.abs() as f64 * 3600.0) as u64;
                    (None, Some(secs))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            batteries.push(BatteryInfo {
                name,
                percentage: capacity,
                state,
                time_to_full,
                time_to_empty,
                health,
                cycle_count: cycles,
            });
        }
    }
    batteries
}

#[cfg(not(target_os = "linux"))]
fn read_batteries() -> Vec<BatteryInfo> {
    // sysinfo's battery info is limited. On macOS we'd need IOKit.
    // On Windows we'd need WMI. For now, return empty.
    Vec::new()
}

fn read_gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    // Try nvidia-smi first (cross-platform, works on Linux and Windows with NVIDIA drivers)
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.total,memory.used,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 5 {
                    let name = parts[0].to_string();
                    let usage = parts[1].parse::<f64>().unwrap_or(0.0);
                    let mem_total = parts[2].parse::<u64>().unwrap_or(0) * 1024 * 1024;
                    let mem_used = parts[3].parse::<u64>().unwrap_or(0) * 1024 * 1024;
                    let temp = parts[4].parse::<f32>().unwrap_or(0.0);
                    gpus.push(GpuInfo {
                        name,
                        usage,
                        temperature: temp,
                        memory_total: mem_total,
                        memory_used: mem_used,
                    });
                }
            }
        }
    }

    // Fallback: try reading from /sys/class/drm/ on Linux
    #[cfg(target_os = "linux")]
    if gpus.is_empty() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm/") {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("card") && !fname.contains('-') {
                    let base = entry.path();
                    let dev = base.join("device");
                    let gpu_name = std::fs::read_to_string(dev.join("product_name")).ok()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| fname.clone());
                    let mem_total = std::fs::read_to_string(dev.join("mem_info_vram_total")).ok()
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(0);
                    let mem_used = std::fs::read_to_string(dev.join("mem_info_vram_used")).ok()
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(0);
                    let temp = find_hwmon_temp(&base.join("device"), "gpu").unwrap_or(0.0);
                    gpus.push(GpuInfo {
                        name: gpu_name,
                        usage: 0.0,
                        temperature: temp,
                        memory_total: mem_total,
                        memory_used: mem_used,
                    });
                }
            }
        }
    }

    gpus
}

#[cfg(target_os = "linux")]
fn find_hwmon_temp(device_path: &std::path::Path, label_filter: &str) -> Option<f32> {
    let hwmon = device_path.join("hwmon");
    if hwmon.exists() {
        if let Ok(entries) = std::fs::read_dir(&hwmon) {
            for entry in entries.flatten() {
                let hwmon_dir = entry.path();
                if let Ok(labels) = std::fs::read_to_string(hwmon_dir.join("temp1_label")) {
                    if labels.trim().to_lowercase().contains(label_filter) {
                        if let Ok(input) = std::fs::read_to_string(hwmon_dir.join("temp1_input")) {
                            if let Ok(millideg) = input.trim().parse::<f32>() {
                                return Some(millideg / 1000.0);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
fn find_hwmon_temp(_device_path: &std::path::Path, _label_filter: &str) -> Option<f32> {
    None
}

// ---------------------------------------------------------------------------
// Thread count helper
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn read_thread_count(pid: sysinfo::Pid) -> u64 {
    use std::fs;
    let path = format!("/proc/{}/status", pid);
    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("Threads:\t") {
                return rest.trim().parse::<u64>().unwrap_or(0);
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
fn read_thread_count(_pid: sysinfo::Pid) -> u64 {
    // Fallback: use tasks count from sysinfo (available on some platforms)
    0
}
