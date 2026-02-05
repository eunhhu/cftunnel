use anyhow::Result;
use std::process::Command;

pub struct ServiceManager {
    name: String,
}

impl ServiceManager {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub fn restart(&self) -> Result<String> {
        let output = Command::new("sudo")
            .args(["systemctl", "restart", &self.name])
            .output()?;

        if output.status.success() {
            Ok("Service restarted successfully".to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to restart: {}", err)
        }
    }

    pub fn status(&self) -> Result<(bool, String)> {
        let active = Command::new("systemctl")
            .args(["is-active", &self.name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let output = Command::new("systemctl")
            .args(["status", &self.name, "--no-pager", "-l"])
            .output()?;

        let status = String::from_utf8_lossy(&output.stdout).to_string();
        Ok((active, status))
    }
}
