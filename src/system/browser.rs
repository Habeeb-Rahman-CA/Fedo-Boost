use super::processes::ProcessInfo;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowserProcessBreakdown {
    pub process_type: String, // "Renderer", "GPU", "Network", "Browser Main"
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowserSummary {
    pub name: String, // "Brave", "Chrome", "Firefox"
    pub total_cpu: f32,
    pub total_memory: u64,
    pub process_count: usize,
    pub breakdowns: Vec<BrowserProcessBreakdown>,
    pub excessive_renderers: Vec<(u32, f32)>, // PID, CPU %
}

pub fn analyze_browser_processes(processes: &[ProcessInfo]) -> Option<BrowserSummary> {
    let browser_procs: Vec<&ProcessInfo> = processes
        .iter()
        .filter(|p| {
            let n = p.name.to_lowercase();
            let c = p.cmd.to_lowercase();
            n.contains("brave") || n.contains("chrome") || n.contains("firefox") ||
            c.contains("brave") || c.contains("chrome") || c.contains("firefox")
        })
        .collect();

    if browser_procs.is_empty() {
        return None;
    }

    let mut total_cpu = 0.0f32;
    let mut total_mem = 0u64;
    let mut renderer_cpu = 0.0f32;
    let mut renderer_mem = 0u64;
    let mut renderer_cnt = 0usize;

    let mut gpu_cpu = 0.0f32;
    let mut gpu_mem = 0u64;
    let mut gpu_cnt = 0usize;

    let mut net_cpu = 0.0f32;
    let mut net_mem = 0u64;
    let mut net_cnt = 0usize;

    let mut main_cpu = 0.0f32;
    let mut main_mem = 0u64;
    let mut main_cnt = 0usize;

    let mut excessive_renderers = Vec::new();

    let browser_name = if browser_procs.iter().any(|p| p.name.to_lowercase().contains("brave")) {
        "Brave".to_string()
    } else if browser_procs.iter().any(|p| p.name.to_lowercase().contains("chrome")) {
        "Chrome".to_string()
    } else {
        "Firefox".to_string()
    };

    for p in &browser_procs {
        total_cpu += p.cpu_usage;
        total_mem += p.memory_bytes;

        let cmd = p.cmd.to_lowercase();
        if cmd.contains("--type=renderer") {
            renderer_cpu += p.cpu_usage;
            renderer_mem += p.memory_bytes;
            renderer_cnt += 1;
            if p.cpu_usage > 25.0 {
                excessive_renderers.push((p.pid, p.cpu_usage));
            }
        } else if cmd.contains("--type=gpu-process") {
            gpu_cpu += p.cpu_usage;
            gpu_mem += p.memory_bytes;
            gpu_cnt += 1;
        } else if cmd.contains("--type=utility") || cmd.contains("network") {
            net_cpu += p.cpu_usage;
            net_mem += p.memory_bytes;
            net_cnt += 1;
        } else {
            main_cpu += p.cpu_usage;
            main_mem += p.memory_bytes;
            main_cnt += 1;
        }
    }

    let mut breakdowns = Vec::new();
    if renderer_cnt > 0 {
        breakdowns.push(BrowserProcessBreakdown {
            process_type: "Renderer".to_string(),
            cpu_usage: renderer_cpu,
            memory_bytes: renderer_mem,
            count: renderer_cnt,
        });
    }
    if gpu_cnt > 0 {
        breakdowns.push(BrowserProcessBreakdown {
            process_type: "GPU".to_string(),
            cpu_usage: gpu_cpu,
            memory_bytes: gpu_mem,
            count: gpu_cnt,
        });
    }
    if net_cnt > 0 {
        breakdowns.push(BrowserProcessBreakdown {
            process_type: "Network".to_string(),
            cpu_usage: net_cpu,
            memory_bytes: net_mem,
            count: net_cnt,
        });
    }
    if main_cnt > 0 {
        breakdowns.push(BrowserProcessBreakdown {
            process_type: "Main".to_string(),
            cpu_usage: main_cpu,
            memory_bytes: main_mem,
            count: main_cnt,
        });
    }

    Some(BrowserSummary {
        name: browser_name,
        total_cpu,
        total_memory: total_mem,
        process_count: browser_procs.len(),
        breakdowns,
        excessive_renderers,
    })
}
