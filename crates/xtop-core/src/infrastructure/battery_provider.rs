use crate::domain::metrics::BatteryInfo;
use crate::domain::system_info::SystemDataProvider;

pub struct NoopBatteryProvider;

impl Default for NoopBatteryProvider {
    fn default() -> Self {
        Self
    }
}

impl NoopBatteryProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SystemDataProvider for NoopBatteryProvider {
    fn refresh_all(&mut self) {}
    fn snapshot(&self) -> crate::domain::metrics::SystemSnapshot {
        unimplemented!("NoopBatteryProvider is meant as a mixin, not a standalone provider")
    }
    fn batteries(&self) -> Vec<BatteryInfo> {
        vec![]
    }
}
