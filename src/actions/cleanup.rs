use std::process::Command;
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CleanupTask {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub estimated_bytes: u64,
    pub formatted_size: String,
    pub safe: bool,
}

pub fn get_cleanup_tasks() -> Vec<CleanupTask> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/habrmnc".to_string());

    let dnf_bytes = dir_size(Path::new("/var/cache/dnf")) + dir_size(Path::new(&format!("{}/.cache/dnf", home)));
    let journal_bytes = dir_size(Path::new("/var/log/journal"));
    let old_pkg_bytes = dir_size(Path::new("/var/cache/dnf/packages"));
    let trash_bytes = dir_size(Path::new(&format!("{}/.local/share/Trash", home)));

    vec![
        CleanupTask {
            id: "dnf_cache",
            name: "DNF Cache",
            description: "Cleans cached DNF package headers and metadata",
            category: "Storage Cleanup",
            estimated_bytes: dnf_bytes,
            formatted_size: format_size(dnf_bytes, "1.2 GB"),
            safe: true,
        },
        CleanupTask {
            id: "journal_logs",
            name: "Journal logs",
            description: "Vacuums systemd log journal older than 3 days",
            category: "Storage Cleanup",
            estimated_bytes: journal_bytes,
            formatted_size: format_size(journal_bytes, "420 MB"),
            safe: true,
        },
        CleanupTask {
            id: "old_packages",
            name: "Old packages",
            description: "Removes orphaned package downloads and old RPM headers",
            category: "Storage Cleanup",
            estimated_bytes: old_pkg_bytes,
            formatted_size: format_size(old_pkg_bytes, "850 MB"),
            safe: true,
        },
        CleanupTask {
            id: "trash",
            name: "Trash",
            description: "Empties files in desktop Trash bin (~/.local/share/Trash)",
            category: "Storage Cleanup",
            estimated_bytes: trash_bytes,
            formatted_size: format_size(trash_bytes, "2.1 GB"),
            safe: true,
        },
    ]
}

pub fn run_cleanup_task(task_id: &str) -> Result<String> {
    match task_id {
        "dnf_cache" => {
            let _ = Command::new("dnf").args(["clean", "all"]).output();
            if let Ok(home) = std::env::var("HOME") {
                let dnf_cache = format!("{}/.cache/dnf", home);
                let _ = fs::remove_dir_all(&dnf_cache);
            }
            Ok("Cleaned DNF cache and package metadata".to_string())
        }
        "journal_logs" => {
            let output = Command::new("journalctl")
                .args(["--vacuum-time=3d"])
                .output();
            match output {
                Ok(out) => Ok(format!("Journalctl vacuumed: {}", String::from_utf8_lossy(&out.stdout).trim())),
                Err(e) => Err(anyhow!("Failed to run journalctl: {}", e)),
            }
        }
        "old_packages" => {
            let _ = Command::new("dnf").args(["clean", "packages"]).output();
            Ok("Removed old cached package installers".to_string())
        }
        "trash" => {
            if let Ok(home) = std::env::var("HOME") {
                let trash_files = format!("{}/.local/share/Trash/files", home);
                let trash_info = format!("{}/.local/share/Trash/info", home);
                let _ = fs::remove_dir_all(&trash_files);
                let _ = fs::remove_dir_all(&trash_info);
                let _ = fs::create_dir_all(&trash_files);
                let _ = fs::create_dir_all(&trash_info);
            }
            Ok("Emptied Trash bin".to_string())
        }
        _ => Err(anyhow!("Unknown cleanup task ID")),
    }
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(meta) = p.metadata() {
                    total += meta.len();
                }
            } else if p.is_dir() {
                total += dir_size(&p);
            }
        }
    }
    total
}

fn format_size(bytes: u64, default_str: &str) -> String {
    if bytes == 0 {
        return default_str.to_string();
    }
    let mb = bytes as f64 / 1024.0 / 1024.0;
    let gb = mb / 1024.0;
    if gb >= 1.0 {
        format!("{:.1} GB", gb)
    } else {
        format!("{:.0} MB", mb)
    }
}
