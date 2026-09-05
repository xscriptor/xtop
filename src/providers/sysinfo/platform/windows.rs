//! Windows probes.
//!
//! Real implementations would use WMI / PowerShell for batteries,
//! `GetAdaptersAddresses` for interface addresses and
//! `NtQueryInformationProcess` for thread counts. Until then the sysinfo
//! crate is the only data source and these helpers stay empty.

use std::collections::HashMap;
use xtop_plugin_api::model::{BatteryInfo, GpuInfo};

pub fn read_cpu_governor(_cpu_id: usize) -> String {
    String::new()
}

/// Per-core temperatures: stub. Windows keeps `CpuInfo::temp_c` at `None`
/// (per-core sensors would come from WMI `MSAcpi_ThermalZoneTemperature` /
/// `MSAcpi_ThermalZone`); the aggregate `SystemSnapshot::cpu_temp` still
/// flows from sysinfo `Components`.
pub fn read_core_temps(logical_cpus: usize) -> Vec<Option<f32>> {
    vec![None; logical_cpus]
}

pub fn read_mount_options() -> HashMap<String, String> {
    HashMap::new()
}

pub fn read_interface_ips() -> HashMap<String, Vec<String>> {
    HashMap::new()
}

pub fn read_batteries() -> Vec<BatteryInfo> {
    Vec::new()
}

pub fn read_thread_count(_pid: sysinfo::Pid) -> u64 {
    0
}

/// Directory Services users: not applicable on Windows; the uid table stays
/// empty and renderers fall back to the numeric uid.
pub fn read_directory_users() -> Vec<(u32, String)> {
    Vec::new()
}

pub fn read_gpu_info_from_sysfs() -> Vec<GpuInfo> {
    Vec::new()
}

/// Package power (RAPL): stub. Windows keeps `SystemInfo::package_power_w`
/// at `None` (a real probe would read the power metering WMI class
/// `MSPower_DeviceEnable`/`Win32_PowerMeter`); widgets hide the readout.
pub struct RaplPower;

impl Default for RaplPower {
    fn default() -> Self {
        Self
    }
}

impl RaplPower {
    pub fn new() -> Self {
        Self
    }

    pub fn sample(&mut self) -> Option<f64> {
        None
    }
}
