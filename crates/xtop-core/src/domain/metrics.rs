#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage: f64,
    pub cpu_id: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub free: u64,
    pub percent: f64,
}

#[derive(Debug, Clone)]
pub struct SwapInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub percent: f64,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub percent: f64,
}

#[derive(Debug, Clone)]
pub struct DiskIOInfo {
    pub name: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_speed: f64,
    pub write_speed: f64,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub rx_speed: f64,
    pub tx_speed: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory: u64,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub percentage: f32,
    pub state: String,
    pub time_to_full: Option<u64>,
    pub time_to_empty: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub usage: f64,
    pub temperature: f32,
    pub memory_total: u64,
    pub memory_used: u64,
}

#[derive(Debug, Clone)]
pub struct DockerInfo {
    pub name: String,
    pub status: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub cpus: Vec<CpuInfo>,
    pub memory: MemoryInfo,
    pub swap: SwapInfo,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub processes: Vec<ProcessInfo>,
    pub load_avg: LoadAvg,
    pub uptime: u64,
    pub cpu_temp: f64,
    pub disk_io: Vec<DiskIOInfo>,
    pub batteries: Vec<BatteryInfo>,
    pub gpus: Vec<GpuInfo>,
    pub dockers: Vec<DockerInfo>,
}
