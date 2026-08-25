use std::fs;
use std::collections::HashMap;
use super::processes::ProcessInfo;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ListeningPortInfo {
    pub port: u16,
    pub process_name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub framework: String,
}

pub fn collect_listening_ports(processes: &[ProcessInfo]) -> Vec<ListeningPortInfo> {
    let mut ports_map: HashMap<u16, ListeningPortInfo> = HashMap::new();

    // Scan /proc/net/tcp and /proc/net/tcp6 for active listening ports
    let listening_ports_set = scan_listening_ports();

    for proc in processes {
        if let Some(port) = proc.safety.listening_port {
            let fw = proc.dev_framework.clone().unwrap_or_else(|| proc.name.clone());
            ports_map.insert(port, ListeningPortInfo {
                port,
                process_name: proc.name.clone(),
                pid: proc.pid,
                cpu_usage: proc.cpu_usage,
                framework: fw,
            });
        }
    }

    // Match listening ports discovered via /proc/net/tcp
    for port in listening_ports_set {
        if !ports_map.contains_key(&port) {
            let label = match port {
                3000 => "Node",
                4200 => "Angular",
                5173 => "Vite",
                5432 => "PostgreSQL",
                6379 => "Redis",
                8095 => "queue_lab_server",
                8080 => "Web Server",
                _ => "Listening Service",
            };
            
            // Find process matching port or framework
            let matching_proc = processes.iter().find(|p| p.cmd.contains(&port.to_string()) || p.name.to_lowercase().contains(&label.to_lowercase()));

            let (pid, name, cpu) = if let Some(p) = matching_proc {
                (p.pid, p.name.clone(), p.cpu_usage)
            } else {
                (0, label.to_string(), 0.0)
            };

            ports_map.insert(port, ListeningPortInfo {
                port,
                process_name: name,
                pid,
                cpu_usage: cpu,
                framework: label.to_string(),
            });
        }
    }

    let mut result: Vec<ListeningPortInfo> = ports_map.into_values().collect();
    result.sort_by_key(|p| p.port);
    result
}

fn scan_listening_ports() -> Vec<u16> {
    let mut ports = Vec::new();
    parse_proc_net("/proc/net/tcp", &mut ports);
    parse_proc_net("/proc/net/tcp6", &mut ports);
    ports.sort();
    ports.dedup();
    ports
}

fn parse_proc_net(path: &str, ports: &mut Vec<u16>) {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let state = parts[3];
                if state == "0A" { // TCP_LISTEN
                    let local_addr = parts[1];
                    if let Some(port_hex) = local_addr.split(':').nth(1) {
                        if let Ok(port) = u16::from_str_radix(port_hex, 16) {
                            if port > 0 {
                                ports.push(port);
                            }
                        }
                    }
                }
            }
        }
    }
}
