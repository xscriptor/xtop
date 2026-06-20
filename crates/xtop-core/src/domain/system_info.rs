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

    /// Downcast to `Any` for internal provider composition.
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Add extra data providers (used by CompositeProvider).
    /// Default no-op implementation for non-composite providers.
    fn add_extras(&mut self, _extras: Vec<Box<dyn SystemDataProvider>>) {}
}
