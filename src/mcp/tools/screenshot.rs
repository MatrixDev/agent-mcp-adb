use crate::adb::device::AdbDevice;
use crate::mcp::tool_result::{ToolResult, failure};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rmcp::model::{CallToolResult, ContentBlock};
use schemars::JsonSchema;
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize, JsonSchema)]
pub struct ScreenshotTool {
    /// Device id as returned by `list_devices`.
    pub device: u64,
}

impl ScreenshotTool {
    ////////////////////////////////////////////////////////////////////////////////
    pub async fn exec(self, device: &AdbDevice) -> ToolResult {
        let png = device.screenshot().await.map_err(failure)?;

        let image = ContentBlock::image(STANDARD.encode(png), "image/png");
        Ok(CallToolResult::success(vec![image]))
    }
}
