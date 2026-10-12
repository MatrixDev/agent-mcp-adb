pub mod device;
pub mod device_info;

use crate::adb::device_info::DeviceInfo;
use anyhow::{Context, bail};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;

////////////////////////////////////////////////////////////////////////////////
const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct Adb {
    path: Arc<PathBuf>,
}

impl Adb {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn new(path: PathBuf) -> Self {
        Self { path: Arc::new(path) }
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn check(&self) -> anyhow::Result<()> {
        self.exec(&["version"]).await?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn exec(&self, args: &[&str]) -> anyhow::Result<Vec<u8>> {
        let mut command = Command::new(self.path.as_ref());
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let output = tokio::time::timeout(COMMAND_TIMEOUT, command.output())
            .await
            .with_context(|| format!("adb {} timed out", args.join(" ")))?
            .with_context(|| format!("failed to run {}", self.path.display()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy_owned(output.stderr);
            bail!("adb {} failed: {}", args.join(" "), stderr.trim());
        }
        Ok(output.stdout)
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn devices(&self) -> anyhow::Result<Vec<DeviceInfo>> {
        let output = self.exec(&["devices", "-l"]).await?;
        Ok(DeviceInfo::parse_list(&String::from_utf8_lossy(&output)))
    }
}
