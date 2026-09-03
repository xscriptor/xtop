//! Cross-platform implementation of [`SystemDataProvider`] for the kernel.
//!
//! Real-time data comes from the `sysinfo` crate; only the OS-specific gaps
//! (governors, batteries, interface IPs, thread counts, extra GPUs) are
//! delegated to [`super::platform`].

use super::platform::shared::read_gpu_info_nvidia_smi;
use super::platform::{
    read_batteries, read_cpu_governor, read_gpu_info_from_sysfs, read_interface_ips,
    read_mount_options, read_thread_count,
};
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind,
    RefreshKind, Signal, System,
};
use xtop_plugin_api::model::*;
use xtop_plugin_api::SystemDataProvider;

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
                let run = if start > 0 {
                    now.saturating_sub(start)
                } else {
                    0
                };
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().to_string(),
                    cpu_usage: p.cpu_usage() as f64,
                    memory: p.memory(),
                    user_id: p.user_id().map(|u| u.to_string()),
                    state: format!("{:?}", p.status()),
                    cmd: p
                        .cmd()
                        .first()
                        .map(|c| c.to_string_lossy().to_string())
                        .unwrap_or_default(),

                    // P0
                    exe_path: p.exe().map(|e| e.to_string_lossy().to_string()),
                    parent_pid: p.parent().map(|ppid| ppid.as_u32()),
                    cmd_full: p
                        .cmd()
                        .iter()
                        .map(|c| c.to_string_lossy().to_string())
                        .collect(),

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
                    environ: p
                        .environ()
                        .iter()
                        .map(|e| e.to_string_lossy().to_string())
                        .collect(),
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

fn read_gpu_info() -> Vec<GpuInfo> {
    // nvidia-smi is shared by Linux and Windows; the remaining platforms get
    // their own fallback probe under platform/.
    let mut gpus = read_gpu_info_nvidia_smi();

    // Fallback: platform-specific detection (e.g. /sys/class/drm on Linux).
    if gpus.is_empty() {
        gpus.extend(read_gpu_info_from_sysfs());
    }

    gpus
}
