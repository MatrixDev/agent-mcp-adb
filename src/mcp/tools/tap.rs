use crate::adb::device::AdbDevice;
use crate::mcp::tool_result::{ToolResult, failure, success};
use schemars::JsonSchema;
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize, JsonSchema)]
pub struct TapTool {
    /// Device id as returned by `list_devices`.
    pub device: u64,
    /// Screen X coordinate in device pixels.
    pub x: u32,
    /// Screen Y coordinate in device pixels.
    pub y: u32,
}

impl TapTool {
    ////////////////////////////////////////////////////////////////////////////////
    pub async fn exec(self, device: &AdbDevice) -> ToolResult {
        device.tap(self.x, self.y).await.map_err(failure)?;
        success(format!("tapped at ({}, {})", self.x, self.y))
    }
}
