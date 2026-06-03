use crate::domain::metrics::*;
use crate::domain::system_info::SystemDataProvider;
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind,
    RefreshKind, System,
};

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

        let disks: Vec<DiskInfo> = self
            .disks
            .iter()
            .map(|d| {
                let total = d.total_space();
                let available = d.available_space();
                let used = total - available;
                DiskInfo {
                    mount_point: d.mount_point().to_string_lossy().to_string(),
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    percent: if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect();

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
                }
            })
            .collect();

        let mut procs: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage: p.cpu_usage() as f64,
                memory: p.memory(),
                user_id: p.user_id().map(|u| u.to_string()),
            })
            .collect();
        procs.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        procs.truncate(50);

        let mut max_temp = 0.0f32;
        for component in &self.components {
            let label = component.label().to_lowercase();
            if label.contains("core") || label.contains("cpu") {
                if let Some(temp) = component.temperature() {
                    if temp > max_temp {
                        max_temp = temp;
                    }
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
            disk_io: vec![],
            batteries: vec![],
            gpus: vec![],
            dockers: vec![],
        }
    }

    fn disk_io(&self) -> Vec<DiskIOInfo> {
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
