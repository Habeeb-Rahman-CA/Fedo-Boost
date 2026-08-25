use std::process::Command;
use anyhow::{Result, anyhow};

pub fn stop_process(pid: u32, force: bool) -> Result<String> {
    let signal = if force { "-9" } else { "-15" };
    let output = Command::new("kill")
        .arg(signal)
        .arg(pid.to_string())
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(format!("Successfully sent signal {} to process {}", signal, pid))
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(anyhow!("Failed to kill PID {}: {}", pid, err.trim()))
            }
        }
        Err(e) => Err(anyhow!("Failed to execute kill command: {}", e)),
    }
}
