#![allow(clippy::manual_non_exhaustive)]

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage: f64,
    pub cpu_id: usize,
    pub frequency: u64,
    pub governor: String,
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
    pub file_system: String,
    pub mount_options: String,
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
    pub ip: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory: u64,
    pub user_id: Option<String>,
    pub state: String,
    pub cmd: String,

    // P0 -- Malicious process detection essentials
    /// Full path to the executable on disk
    pub exe_path: Option<String>,
    /// Parent process ID
    pub parent_pid: Option<u32>,
    /// Full command-line argument vector (argv)
    pub cmd_full: Vec<String>,

    // P1 -- Timing, privilege, and context
    /// Process start time as epoch seconds
    pub start_time: u64,
    /// Seconds since process started
    pub run_time: u64,
    /// Effective user ID (may differ from uid on SUID binaries)
    pub effective_user_id: Option<String>,
    /// Group ID
    pub group_id: Option<String>,
    /// Process working directory
    pub cwd: Option<String>,
    /// Number of threads
    pub thread_count: u64,

    // P2 -- I/O, environment, session
    /// Number of open file descriptors
    pub open_files: u64,
    /// Max allowed file descriptors
    pub open_files_limit: u64,
    /// Total bytes read from disk by this process
    pub disk_total_read_bytes: u64,
    /// Total bytes written to disk by this process
    pub disk_total_write_bytes: u64,
    /// Environment variables
    pub environ: Vec<String>,
    /// Session ID
    pub session_id: Option<u32>,
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
    pub health: f32,
    pub cycle_count: Option<u64>,
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

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_version: String,
    pub kernel: String,
    pub desktop_env: String,
    pub shell: String,
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
    pub sys_info: SystemInfo,
}
