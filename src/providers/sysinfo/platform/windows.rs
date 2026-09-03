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

pub fn read_gpu_info_from_sysfs() -> Vec<GpuInfo> {
    Vec::new()
}
