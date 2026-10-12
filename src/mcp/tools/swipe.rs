use crate::adb::device::AdbDevice;
use crate::mcp::tool_result::{ToolResult, failure, success};
use schemars::JsonSchema;
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize, JsonSchema)]
pub struct SwipeTool {
    /// Device id as returned by `list_devices`.
    pub device: u64,
    /// Start X coordinate in device pixels.
    pub x1: u32,
    /// Start Y coordinate in device pixels.
    pub y1: u32,
    /// End X coordinate in device pixels.
    pub x2: u32,
    /// End Y coordinate in device pixels.
    pub y2: u32,
    /// Swipe duration in milliseconds; longer swipes scroll with less inertia.
    pub duration_ms: Option<u32>,
}

impl SwipeTool {
    ////////////////////////////////////////////////////////////////////////////////
    pub async fn exec(self, device: &AdbDevice) -> ToolResult {
        let (from, to) = ((self.x1, self.y1), (self.x2, self.y2));
        let duration_ms = self.duration_ms.unwrap_or(300);

        device.swipe(from, to, duration_ms).await.map_err(failure)?;

        success(format!("swiped from {from:?} to {to:?} in {duration_ms}ms"))
    }
}
