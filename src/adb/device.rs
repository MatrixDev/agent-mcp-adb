use crate::adb::{Adb, DeviceInfo};
use anyhow::bail;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};

////////////////////////////////////////////////////////////////////////////////
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const UI_DUMP_FILE: &str = "/data/local/tmp/agent-mcp-adb-ui-dump.xml";
const HIERARCHY_END: &str = "</hierarchy>";

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct AdbDevice {
    id: u64,
    adb: Adb,
    info: DeviceInfo,
}

impl AdbDevice {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn new(adb: Adb, info: DeviceInfo) -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { id, adb, info }
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn id(&self) -> u64 {
        self.id
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn details(&self) -> String {
        let mut details = vec![format!("serial: {}", self.serial)];
        if let Some(product) = &self.product {
            details.push(format!("product: {product}"));
        }
        if let Some(model) = &self.model {
            details.push(format!("model: {model}"));
        }
        details.join(", ")
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn screenshot(&self) -> anyhow::Result<Vec<u8>> {
        let png = self.exec_out("screencap -p").await?;
        if !png.starts_with(PNG_SIGNATURE) {
            bail!("screencap returned no image: {}", String::from_utf8_lossy(&png).trim());
        }
        Ok(png)
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn ui_dump(&self, compressed: bool) -> anyhow::Result<String> {
        let flags = if compressed { "--compressed" } else { "" };

        // dump straight to stdout without touching the filesystem
        let output = self.exec_out(&format!("uiautomator dump {flags} /dev/tty")).await?;
        if let Some(xml) = extract_hierarchy(&output) {
            return Ok(xml);
        }

        // some devices can't write to /dev/tty, fall back to a temporary file
        let command =
            format!("uiautomator dump {flags} {UI_DUMP_FILE} >/dev/null && cat {UI_DUMP_FILE}; rm -f {UI_DUMP_FILE}");
        let output = self.exec_out(&command).await?;
        match extract_hierarchy(&output) {
            Some(xml) => Ok(xml),
            None => bail!("uiautomator dump failed: {}", String::from_utf8_lossy(&output).trim()),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn tap(&self, x: u32, y: u32) -> anyhow::Result<()> {
        self.shell(&format!("input tap {x} {y}")).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn swipe(&self, from: (u32, u32), to: (u32, u32), duration_ms: u32) -> anyhow::Result<()> {
        let ((x1, y1), (x2, y2)) = (from, to);
        let command = format!("input swipe {x1} {y1} {x2} {y2} {duration_ms}");
        self.shell(&command).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    async fn exec_out(&self, command: &str) -> anyhow::Result<Vec<u8>> {
        self.adb.exec(&["-s", &self.serial, "exec-out", command]).await
    }

    ////////////////////////////////////////////////////////////////////////////////
    async fn shell(&self, command: &str) -> anyhow::Result<()> {
        self.adb.exec(&["-s", &self.serial, "shell", command]).await?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
impl Deref for AdbDevice {
    type Target = DeviceInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

////////////////////////////////////////////////////////////////////////////////
fn extract_hierarchy(output: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(output);
    let start = text.find("<?xml").or_else(|| text.find("<hierarchy"))?;
    let end = text.rfind(HIERARCHY_END)? + HIERARCHY_END.len();
    (start < end).then(|| text[start..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn extracts_hierarchy_from_tty_dump() {
        let output = b"<?xml version='1.0' encoding='UTF-8' standalone='yes' ?><hierarchy rotation=\"0\"></hierarchy>UI hierchary dumped to: /dev/tty\r\n";
        let xml = extract_hierarchy(output);
        assert_eq!(
            xml.as_deref(),
            Some("<?xml version='1.0' encoding='UTF-8' standalone='yes' ?><hierarchy rotation=\"0\"></hierarchy>")
        );
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn rejects_failed_dump() {
        assert_eq!(extract_hierarchy(b"ERROR: could not get idle state.\n"), None);
    }
}
