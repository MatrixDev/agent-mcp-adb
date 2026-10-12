mod adb;
mod context;
mod mcp;
mod tui;

use crate::adb::Adb;
use crate::context::Context;
use crate::mcp::McpServer;
use crate::tui::picker::DevicePicker;
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::select;

////////////////////////////////////////////////////////////////////////////////
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// MCP server IPv4/IPv6 address
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
    address: IpAddr,

    /// MCP server port (random free port if not set)
    #[arg(short, long)]
    port: Option<u16>,

    /// Path to the adb executable
    #[arg(long, default_value = "adb")]
    adb: PathBuf,
}

////////////////////////////////////////////////////////////////////////////////
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let adb = Adb::new(args.adb);
    adb.check().await?;
    let context = Context::default();

    let listener = TcpListener::bind((args.address, args.port.unwrap_or(0))).await?;
    let url = format!("http://{}/mcp", listener.local_addr()?);

    let server = McpServer::new(context.clone());
    let picker = DevicePicker::new(context, url);

    select! {
        e = server.run(listener) => e?,
        e = picker.run(&adb) => e?,
    }
    Ok(())
}
