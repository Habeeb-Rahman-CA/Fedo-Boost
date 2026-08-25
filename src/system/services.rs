use std::process::Command;
use super::processes::ProcessInfo;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub active_state: String, // active, inactive, failed
    pub sub_state: String,    // running, exited, dead
    pub load_state: String,   // loaded, not-found
    pub is_enabled: bool,
    pub is_potentially_unnecessary: bool,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub description: String,
}

pub struct ServiceCollector;

impl ServiceCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self, processes: &[ProcessInfo]) -> Vec<ServiceInfo> {
        let mut services = Vec::new();

        let key_services = [
            ("docker.service", "Docker", "Docker Application Container Engine", true),
            ("redis.service", "Redis", "Redis In-Memory Data Structure Store", false),
            ("postgresql.service", "PostgreSQL", "PostgreSQL Database Server", true),
            ("bluetooth.service", "Bluetooth", "Bluetooth Protocol Stack", false),
            ("packagekit.service", "PackageKit", "PackageKit DBus Service", false),
            ("cups.service", "CUPS", "CUPS Printing Scheduler", true),
            ("sshd.service", "OpenSSH", "OpenSSH Server Daemon", true),
            ("firewalld.service", "Firewalld", "Firewall Daemon", false),
            ("NetworkManager.service", "NetworkManager", "Network Manager", false),
        ];

        for (svc_name, display_name, desc, unnecessary_candidate) in key_services {
            let output = Command::new("systemctl")
                .args(["is-active", svc_name])
                .output();

            let active_state = match output {
                Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
                Err(_) => "unknown".to_string(),
            };

            let enabled_output = Command::new("systemctl")
                .args(["is-enabled", svc_name])
                .output();

            let is_enabled = match enabled_output {
                Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "enabled",
                Err(_) => false,
            };

            let (cpu, mem) = Self::estimate_service_resources(display_name, processes);

            let is_potentially_unnecessary = unnecessary_candidate && active_state == "active";

            services.push(ServiceInfo {
                name: svc_name.to_string(),
                display_name: display_name.to_string(),
                active_state: active_state.clone(),
                sub_state: if active_state == "active" { "running".to_string() } else { "stopped".to_string() },
                load_state: "loaded".to_string(),
                is_enabled,
                is_potentially_unnecessary,
                cpu_usage: cpu,
                memory_bytes: mem,
                description: desc.to_string(),
            });
        }

        services
    }

    fn estimate_service_resources(display_name: &str, processes: &[ProcessInfo]) -> (f32, u64) {
        let needle = display_name.to_lowercase();
        let mut total_cpu = 0.0f32;
        let mut total_mem = 0u64;

        for proc in processes {
            let n = proc.name.to_lowercase();
            let c = proc.cmd.to_lowercase();
            if n.contains(&needle) || c.contains(&needle) || (needle == "docker" && (n.contains("dockerd") || n.contains("containerd"))) {
                total_cpu += proc.cpu_usage;
                total_mem += proc.memory_bytes;
            }
        }

        (total_cpu, total_mem)
    }
}
