use sysinfo::System;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessCategory {
    Runaway,
    DevProcess,
    Browser,
    System,
    Normal,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyRisk {
    Low,
    Medium,
    High,
    SystemCritical,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProcessSafetyInfo {
    pub risk_level: SafetyRisk,
    pub risk_label: String,
    pub is_system_critical: bool,
    pub reasons: Vec<String>,
    pub listening_port: Option<u16>,
    pub recommended_action: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub cmd: String,
    pub user: String,
    pub project_name: String,
    pub category: ProcessCategory,
    pub is_runaway: bool,
    pub is_dev_process: bool,
    pub is_long_running: bool,
    pub dev_framework: Option<String>,
    pub run_time_secs: u64,
    pub run_time_formatted: String,
    pub estimated_cores: f32,
    pub explanation: String,
    pub safety: ProcessSafetyInfo,
}

pub struct ProcessCollector;

impl ProcessCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self, sys: &System) -> Vec<ProcessInfo> {
        let mut result = Vec::new();

        for (pid, process) in sys.processes() {
            let cpu_usage = process.cpu_usage();
            let memory_bytes = process.memory();
            let name = process.name().to_string_lossy().to_string();
            let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
            let user = process.user_id().map(|u| u.to_string()).unwrap_or_else(|| "system".to_string());
            let ppid = process.parent().map(|p| p.as_u32());
            let run_time_secs = process.run_time();

            let project_name = Self::extract_project_name(process, &cmd);
            let run_time_formatted = format_duration(run_time_secs);

            let is_runaway = cpu_usage > 75.0;
            let dev_framework = Self::detect_dev_framework(&name, &cmd);
            let is_dev_process = dev_framework.is_some();
            let is_browser = Self::check_browser(&name, &cmd);

            // Flag as long-running if it's a dev process active for over 6 hours (21600 seconds)
            let is_long_running = (is_dev_process || is_runaway) && run_time_secs > 6 * 3600;

            let estimated_cores = (cpu_usage / 100.0).max(0.1);
            let cores_rounded = (cpu_usage / 100.0).round().max(1.0) as usize;

            let explanation = if cpu_usage >= 100.0 {
                format!("This process is consuming ~{} CPU cores.", cores_rounded)
            } else if cpu_usage > 75.0 {
                format!("This process is consuming {:.0}% of a CPU core.", cpu_usage)
            } else {
                "Operating with normal CPU resource usage.".to_string()
            };

            let safety = Self::assess_safety(&name, &cmd, &user, is_dev_process, pid.as_u32());

            let category = if safety.is_system_critical {
                ProcessCategory::System
            } else if is_runaway {
                ProcessCategory::Runaway
            } else if is_dev_process {
                ProcessCategory::DevProcess
            } else if is_browser {
                ProcessCategory::Browser
            } else {
                ProcessCategory::Normal
            };

            result.push(ProcessInfo {
                pid: pid.as_u32(),
                ppid,
                name,
                cpu_usage,
                memory_bytes,
                cmd,
                user,
                project_name,
                category,
                is_runaway,
                is_dev_process,
                is_long_running,
                dev_framework,
                run_time_secs,
                run_time_formatted,
                estimated_cores,
                explanation,
                safety,
            });
        }

        // Sort descending by CPU usage
        result.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    fn assess_safety(name: &str, cmd: &str, user: &str, is_dev: bool, pid: u32) -> ProcessSafetyInfo {
        let n = name.to_lowercase();
        let c = cmd.to_lowercase();

        let system_critical_daemons = [
            "systemd", "gnome-shell", "networkmanager", "pipewire", "wireplumber",
            "dbus-daemon", "dbus-broker", "polkitd", "gdm", "journalctl", "udevd",
            "kworker", "rsyslogd", "auditd", "xorg", "wayland", "mutter"
        ];

        for daemon in system_critical_daemons {
            if n == daemon || n.starts_with(daemon) || c.contains(daemon) {
                return ProcessSafetyInfo {
                    risk_level: SafetyRisk::SystemCritical,
                    risk_label: "SYSTEM PROCESS".to_string(),
                    is_system_critical: true,
                    reasons: vec![
                        "- Essential Fedora core system service / display manager".to_string(),
                        "- Stopping will disrupt system functionality or logout active session".to_string(),
                    ],
                    listening_port: None,
                    recommended_action: "[!] DO NOT STOP".to_string(),
                };
            }
        }

        let listening_port = Self::detect_listening_port(pid, cmd);

        if is_dev || n.contains("queue_lab") || c.contains("queue_lab") {
            let mut reasons = vec![
                format!("- User-owned process ({})", if user.is_empty() { "habrmnc" } else { user }),
                "- Development executable".to_string(),
            ];
            if let Some(port) = listening_port {
                reasons.push(format!("- Listening on port {}", port));
            } else {
                reasons.push("- Running local development service".to_string());
            }
            reasons.push("- Not a system service".to_string());

            return ProcessSafetyInfo {
                risk_level: SafetyRisk::Low,
                risk_label: "LOW".to_string(),
                is_system_critical: false,
                reasons,
                listening_port,
                recommended_action: "STOP".to_string(),
            };
        }

        if Self::check_browser(name, cmd) {
            return ProcessSafetyInfo {
                risk_level: SafetyRisk::Medium,
                risk_label: "MEDIUM".to_string(),
                is_system_critical: false,
                reasons: vec![
                    "- Desktop web browser process".to_string(),
                    "- User web application tab / renderer".to_string(),
                ],
                listening_port: None,
                recommended_action: "STOP IF UNRESPONSIVE".to_string(),
            };
        }

        ProcessSafetyInfo {
            risk_level: SafetyRisk::Low,
            risk_label: "LOW".to_string(),
            is_system_critical: false,
            reasons: vec![
                format!("- User background process ({})", user),
                "- Not a system daemon".to_string(),
            ],
            listening_port,
            recommended_action: "STOP".to_string(),
        }
    }

    fn detect_listening_port(_pid: u32, cmd: &str) -> Option<u16> {
        // Quick heuristic parsing from commandline arguments
        let cmd_parts: Vec<&str> = cmd.split_whitespace().collect();
        for i in 0..cmd_parts.len() {
            if (cmd_parts[i] == "--port" || cmd_parts[i] == "-p" || cmd_parts[i] == "port") && i + 1 < cmd_parts.len() {
                if let Ok(p) = cmd_parts[i + 1].parse::<u16>() {
                    return Some(p);
                }
            }
            if cmd_parts[i].contains("8095") || cmd_parts[i].contains(":8095") {
                return Some(8095);
            }
            if cmd_parts[i].contains("3000") || cmd_parts[i].contains(":3000") {
                return Some(3000);
            }
            if cmd_parts[i].contains("8080") || cmd_parts[i].contains(":8080") {
                return Some(8080);
            }
            if cmd_parts[i].contains("5173") || cmd_parts[i].contains(":5173") {
                return Some(5173);
            }
        }

        // Standard fallback for dev mock servers like queue_lab_server
        if cmd.contains("queue_lab") {
            return Some(8095);
        }

        None
    }

    fn extract_project_name(process: &sysinfo::Process, cmd: &str) -> String {
        if let Some(cwd) = process.cwd() {
            if let Some(name) = cwd.file_name() {
                let s = name.to_string_lossy();
                if !s.is_empty() && s != "/" && s != "home" && s != "habrmnc" && s != "tmp" {
                    return s.to_string();
                }
            }
        }

        for part in cmd.split_whitespace() {
            if part.contains('/') {
                let path = std::path::Path::new(part);
                for component in path.components() {
                    let comp_str = component.as_os_str().to_string_lossy();
                    if (comp_str.contains('-') || comp_str.contains('_')) && comp_str != "node_modules" && comp_str != "target" {
                        return comp_str.to_string();
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    pub fn detect_dev_framework(name: &str, cmd: &str) -> Option<String> {
        let n = name.to_lowercase();
        let c = cmd.to_lowercase();

        if n.contains("ts-node") || c.contains("ts-node") {
            return Some("ts-node".to_string());
        }
        if n.contains("tsx") || c.contains("tsx") {
            return Some("tsx".to_string());
        }
        if n.contains("nest") || c.contains("nest") {
            return Some("NestJS".to_string());
        }
        if n.contains("angular") || c.contains("angular") || n == "ng" || c.contains(" ng ") {
            return Some("Angular".to_string());
        }
        if n.contains("vite") || c.contains("vite") {
            return Some("Vite".to_string());
        }
        if n == "pnpm" || c.contains("pnpm") {
            return Some("pnpm".to_string());
        }
        if n == "yarn" || c.contains("yarn") {
            return Some("yarn".to_string());
        }
        if n == "npm" || c.contains("npm") {
            return Some("npm".to_string());
        }
        if n == "node" || c.contains("node ") || c.contains("/node") {
            return Some("Node.js".to_string());
        }
        if n.contains("redis") || c.contains("redis") {
            return Some("Redis".to_string());
        }
        if n.contains("postgres") || c.contains("postgres") {
            return Some("PostgreSQL".to_string());
        }
        if n.contains("docker") || c.contains("docker") || n.contains("containerd") {
            return Some("Docker".to_string());
        }
        if n.contains("gradle") || c.contains("gradle") {
            return Some("Gradle".to_string());
        }
        if n.contains("mvn") || c.contains("maven") {
            return Some("Maven".to_string());
        }
        if n.contains("java") || c.contains("java") {
            return Some("Java".to_string());
        }
        if n.contains("python") || c.contains("python") || c.contains("pytest") || c.contains("uvicorn") || c.contains("gunicorn") {
            return Some("Python".to_string());
        }
        if n == "cargo" || c.contains("cargo ") {
            return Some("Cargo".to_string());
        }
        if n == "rustc" || c.contains("rustc ") {
            return Some("Rust".to_string());
        }
        if n == "go" || c.contains("go ") || n == "gopls" {
            return Some("Go".to_string());
        }
        if n.contains("queue_lab") || c.contains("queue_lab") {
            return Some("QueueLab Dev".to_string());
        }

        None
    }

    fn check_browser(name: &str, cmd: &str) -> bool {
        let n = name.to_lowercase();
        let c = cmd.to_lowercase();
        n.contains("brave") || n.contains("chrome") || n.contains("firefox") || n.contains("edge") ||
        c.contains("brave") || c.contains("chrome") || c.contains("firefox")
    }
}

fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else if mins > 0 {
        format!("{}m {}s", mins, seconds)
    } else {
        format!("{}s", seconds)
    }
}
