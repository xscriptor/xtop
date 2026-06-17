use crate::domain::metrics::*;

pub trait SystemDataProvider: Send {
    fn refresh_all(&mut self);
    fn snapshot(&self) -> SystemSnapshot;
    fn disk_io(&self) -> Vec<DiskIOInfo> {
        vec![]
    }
    fn batteries(&self) -> Vec<BatteryInfo> {
        vec![]
    }
    fn gpu_info(&self) -> Vec<GpuInfo> {
        vec![]
    }
    fn docker_info(&self) -> Vec<DockerInfo> {
        vec![]
    }
    fn system_info(&self) -> SystemInfo {
        SystemInfo::default()
    }
    fn kill_process(&self, _pid: u32) -> bool {
        false
    }
}
