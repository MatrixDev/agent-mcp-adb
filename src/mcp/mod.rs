mod tool_result;
mod tools;

use crate::adb::device::AdbDevice;
use crate::context::Context;
use crate::mcp::tool_result::ToolResult;
use crate::mcp::tools::list_devices::ListDevicesTool;
use crate::mcp::tools::screenshot::ScreenshotTool;
use crate::mcp::tools::swipe::SwipeTool;
use crate::mcp::tools::tap::TapTool;
use crate::mcp::tools::ui_dump::UiDumpTool;
use crate::mcp::tools::wait::WaitTool;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, tool, tool_router};
use tokio::net::TcpListener;

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct McpServer {
    context: Context,
}

impl McpServer {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn new(devices: Context) -> Self {
        Self { context: devices }
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn run(self, listener: TcpListener) -> anyhow::Result<()> {
        let service = StreamableHttpService::new(
            move || Ok(self.clone()),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default(),
        );
        let router = axum::Router::new().nest_service("/mcp", service);

        Ok(axum::serve(listener, router).await?)
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn device(&self, id: u64) -> Result<AdbDevice, ErrorData> {
        let device = self.context.get(id);
        device.ok_or_else(|| ErrorData::invalid_params(format!("device {id} is not exposed"), None))
    }
}

#[tool_router(server_handler)]
impl McpServer {
    ////////////////////////////////////////////////////////////////////////////////
    #[tool(description = "List devices available for control, with their ids, names and adb serials")]
    async fn list_devices(&self, Parameters(tool): Parameters<ListDevicesTool>) -> ToolResult {
        tool.exec(self.context.all()).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[tool(
        description = "Capture a PNG screenshot of a device screen. Apps with secure content may come out black, use `ui_dump` for them"
    )]
    async fn screenshot(&self, Parameters(tool): Parameters<ScreenshotTool>) -> ToolResult {
        let device = self.device(tool.device)?;
        tool.exec(&device).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[tool(
        description = "Dump the current UI hierarchy of a device as XML (`uiautomator dump`), with texts, resource ids and screen bounds of views"
    )]
    async fn ui_dump(&self, Parameters(tool): Parameters<UiDumpTool>) -> ToolResult {
        let device = self.device(tool.device)?;
        tool.exec(&device).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[tool(description = "Wait for the given number of milliseconds")]
    async fn wait(&self, Parameters(tool): Parameters<WaitTool>) -> ToolResult {
        tool.exec().await
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[tool(description = "Tap at screen coordinates (device pixels, same as screenshot and ui_dump bounds)")]
    async fn tap(&self, Parameters(tool): Parameters<TapTool>) -> ToolResult {
        let device = self.device(tool.device)?;
        tool.exec(&device).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[tool(description = "Swipe between two screen points (device pixels), e.g. to scroll content")]
    async fn swipe(&self, Parameters(tool): Parameters<SwipeTool>) -> ToolResult {
        let device = self.device(tool.device)?;
        tool.exec(&device).await
    }
}
