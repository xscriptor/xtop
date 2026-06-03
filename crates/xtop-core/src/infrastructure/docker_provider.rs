use crate::domain::metrics::DockerInfo;
use crate::domain::system_info::SystemDataProvider;

pub struct NoopDockerProvider;

impl Default for NoopDockerProvider {
    fn default() -> Self {
        Self
    }
}

impl NoopDockerProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SystemDataProvider for NoopDockerProvider {
    fn refresh_all(&mut self) {}
    fn snapshot(&self) -> crate::domain::metrics::SystemSnapshot {
        unimplemented!("NoopDockerProvider is meant as a mixin, not a standalone provider")
    }
    fn docker_info(&self) -> Vec<DockerInfo> {
        vec![]
    }
}
