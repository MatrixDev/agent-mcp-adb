use crate::adb::device::AdbDevice;
use crate::mcp::tool_result::{ToolResult, failure, success};
use schemars::JsonSchema;
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize, JsonSchema)]
pub struct UiDumpTool {
    /// Device id as returned by `list_devices`.
    pub device: u64,
    /// Skip views that are not important for accessibility (smaller output).
    #[serde(default)]
    pub compressed: bool,
}

impl UiDumpTool {
    ////////////////////////////////////////////////////////////////////////////////
    pub async fn exec(self, device: &AdbDevice) -> ToolResult {
        let xml = device.ui_dump(self.compressed).await.map_err(failure)?;
        success(xml)
    }
}
