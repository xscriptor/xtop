use crate::domain::metrics::GpuInfo;
use crate::domain::system_info::SystemDataProvider;

pub struct NoopGpuProvider;

impl Default for NoopGpuProvider {
    fn default() -> Self {
        Self
    }
}

impl NoopGpuProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SystemDataProvider for NoopGpuProvider {
    fn refresh_all(&mut self) {}
    fn snapshot(&self) -> crate::domain::metrics::SystemSnapshot {
        unimplemented!("NoopGpuProvider is meant as a mixin, not a standalone provider")
    }
    fn gpu_info(&self) -> Vec<GpuInfo> {
        vec![]
    }
}
