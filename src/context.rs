use crate::adb::device::AdbDevice;
use parking_lot::RwLock;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Default)]
pub struct Context {
    devices: Arc<RwLock<Vec<AdbDevice>>>,
}

impl Context {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn set(&self, devices: Vec<AdbDevice>) {
        *self.devices.write() = devices;
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn all(&self) -> Vec<AdbDevice> {
        self.devices.read().clone()
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn get(&self, id: u64) -> Option<AdbDevice> {
        let devices = self.devices.read();
        devices.iter().find(|device| device.id() == id).cloned()
    }
}
