use crate::adb::device::AdbDevice;
use crate::mcp::tool_result::{ToolResult, success};
use schemars::JsonSchema;
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize, JsonSchema)]
pub struct ListDevicesTool {}

impl ListDevicesTool {
    ////////////////////////////////////////////////////////////////////////////////
    pub async fn exec(self, devices: Vec<AdbDevice>) -> ToolResult {
        if devices.is_empty() {
            return success("no devices are exposed");
        }

        let lines: Vec<_> = devices
            .iter()
            .map(|device| format!("{}: {} ({})", device.id(), device.name.as_str(), device.details()))
            .collect();

        success(lines.join("\n"))
    }
}
