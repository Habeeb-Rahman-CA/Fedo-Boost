use std::process::Command;
use anyhow::{Result, anyhow};

pub fn toggle_service(name: &str, current_state: &str) -> Result<String> {
    let action = if current_state == "active" { "stop" } else { "start" };
    let output = Command::new("systemctl")
        .arg(action)
        .arg(name)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(format!("Service {} {}ed successfully", name, action))
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(anyhow!("systemctl {} {} failed: {}", action, name, err.trim()))
            }
        }
        Err(e) => Err(anyhow!("Failed to run systemctl: {}", e)),
    }
}

#[allow(dead_code)]
pub fn restart_service(name: &str) -> Result<String> {
    let output = Command::new("systemctl")
        .arg("restart")
        .arg(name)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(format!("Service {} restarted successfully", name))
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(anyhow!("systemctl restart {} failed: {}", name, err.trim()))
            }
        }
        Err(e) => Err(anyhow!("Failed to run systemctl: {}", e)),
    }
}
