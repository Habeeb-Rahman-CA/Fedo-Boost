use crate::system::SystemSnapshot;
use crate::diagnosis::rules::{DiagnosticIssue, IssueSeverity};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiagnosisReport {
    pub health_score: u8,
    pub primary_status: String,
    pub issues: Vec<DiagnosticIssue>,
    pub dev_processes_summary: Vec<String>,
    pub cpu_healthy: bool,
    pub memory_healthy: bool,
    pub storage_healthy: bool,
    pub swap_healthy: bool,
}

pub struct SystemAnalyzer;

impl SystemAnalyzer {
    pub fn analyze(snapshot: &SystemSnapshot) -> DiagnosisReport {
        let mut issues = Vec::new();
        let mut penalty = 0u8;

        // 1. CPU Check & Runaway Processes
        let cpu_healthy = snapshot.cpu.overall_usage < 75.0;
        let mut runaway_count = 0;
        for proc in &snapshot.processes {
            if proc.is_runaway {
                runaway_count += 1;
                issues.push(DiagnosticIssue {
                    severity: IssueSeverity::Critical,
                    category: "CPU".to_string(),
                    title: format!("Runaway Process: {} (PID {})", proc.name, proc.pid),
                    description: format!("Consuming {:.0}% CPU (Running for {})", proc.cpu_usage, proc.run_time_formatted),
                    recommendation: format!("Stop process PID {}", proc.pid),
                    fixable_action: Some(format!("kill:{}", proc.pid)),
                });
            } else if proc.is_long_running {
                issues.push(DiagnosticIssue {
                    severity: IssueSeverity::Warning,
                    category: "DEV".to_string(),
                    title: format!("[!] Long-running development process: {} (PID {})", proc.name, proc.pid),
                    description: format!("Running for {} - may be a forgotten dev server", proc.run_time_formatted),
                    recommendation: format!("Stop process PID {} if no longer active", proc.pid),
                    fixable_action: Some(format!("kill:{}", proc.pid)),
                });
            }
        }

        if !cpu_healthy && runaway_count == 0 {
            penalty += 25;
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Warning,
                category: "CPU".to_string(),
                title: "High System CPU Usage".to_string(),
                description: format!("Overall CPU is at {:.1}%", snapshot.cpu.overall_usage),
                recommendation: "Investigate top active background tasks".to_string(),
                fixable_action: None,
            });
        } else if runaway_count > 0 {
            penalty += (runaway_count * 20).min(50) as u8;
        }

        // 2. Memory & ZRAM Check (accounting for Linux Cache vs True Available RAM)
        let memory_healthy = !snapshot.memory.is_under_pressure;
        if !memory_healthy {
            penalty += 20;
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Warning,
                category: "RAM".to_string(),
                title: "High Memory Pressure".to_string(),
                description: format!("Memory pressure is at {:.1}% ({:.1} GB available)", snapshot.memory.memory_pressure, snapshot.memory.available_bytes as f64 / 1e9),
                recommendation: "Close unneeded heavy applications to release memory".to_string(),
                fixable_action: None,
            });
        }

        let swap_healthy = snapshot.swap.usage_percent < 50.0;
        if !swap_healthy {
            penalty += 15;
            let title = if snapshot.swap.is_zram {
                "Active ZRAM Swap Pressure"
            } else {
                "High Swap Memory Usage"
            };
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Warning,
                category: "ZRAM".to_string(),
                title: title.to_string(),
                description: format!("Swap usage is at {:.1}%", snapshot.swap.usage_percent),
                recommendation: "Free memory to reduce ZRAM swapping and CPU compression overhead".to_string(),
                fixable_action: None,
            });
        }

        // 3. Storage Check
        let mut storage_healthy = true;
        for disk in &snapshot.disks {
            if disk.usage_percent > 90.0 {
                storage_healthy = false;
                penalty += 20;
                issues.push(DiagnosticIssue {
                    severity: IssueSeverity::Critical,
                    category: "DISK".to_string(),
                    title: format!("Low Disk Space on {}", disk.mount_point),
                    description: format!("{:.1}% used ({:.1} GB available)", disk.usage_percent, disk.available_bytes as f64 / 1e9),
                    recommendation: "Run Fedora cache cleanup (DNF / Journal logs)".to_string(),
                    fixable_action: Some("cleanup:all".to_string()),
                });
            }
        }

        // 4. Developer Processes Tree Hierarchy
        let mut project_groups: std::collections::BTreeMap<String, Vec<&crate::system::ProcessInfo>> = std::collections::BTreeMap::new();

        for proc in &snapshot.processes {
            if proc.is_dev_process {
                project_groups.entry(proc.project_name.clone()).or_default().push(proc);
            }
        }

        let mut dev_processes_summary = Vec::new();
        for (project, procs) in project_groups {
            dev_processes_summary.push(format!("v {}", project));
            
            // Build tree lines for processes in this project
            for (idx, p) in procs.iter().enumerate() {
                let cmd_short = if p.cmd.len() > 30 { format!("{}...", &p.cmd[..27]) } else if !p.cmd.is_empty() { p.cmd.clone() } else { p.name.clone() };
                let indent = if idx == 0 { "   |- " } else { "      |- " };
                dev_processes_summary.push(format!("{}{}", indent, cmd_short));
            }
        }

        // 5. Determine Primary Status string
        let primary_status = if snapshot.cpu.is_saturated {
            "[!] CPU saturation".to_string()
        } else if runaway_count > 0 {
            "[!] High CPU usage (Runaway Process)".to_string()
        } else if !cpu_healthy {
            "[!] High CPU usage".to_string()
        } else if !memory_healthy {
            "[!] Memory pressure".to_string()
        } else if !swap_healthy {
            "[!] ZRAM pressure detected".to_string()
        } else if !storage_healthy {
            "[!] Storage almost full".to_string()
        } else {
            "[OK] System healthy".to_string()
        };

        let health_score = 100u8.saturating_sub(penalty);

        DiagnosisReport {
            health_score,
            primary_status,
            issues,
            dev_processes_summary,
            cpu_healthy,
            memory_healthy,
            storage_healthy,
            swap_healthy,
        }
    }
}
