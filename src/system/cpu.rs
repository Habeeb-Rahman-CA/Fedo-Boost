use sysinfo::System;
use std::fs;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub overall_usage: f32,
    pub user_percent: f32,
    pub system_percent: f32,
    pub iowait_percent: f32,
    pub idle_percent: f32,
    pub core_usages: Vec<f32>,
    pub core_count: usize,
    pub brand: String,
    pub load_avg_1m: f64,
    pub load_avg_5m: f64,
    pub load_avg_15m: f64,
    pub is_saturated: bool,
}

pub fn collect_cpu(sys: &System) -> CpuMetrics {
    let cpus = sys.cpus();
    let overall_usage = sys.global_cpu_usage();
    let idle_percent = (100.0 - overall_usage).max(0.0);
    let core_usages = cpus.iter().map(|c| c.cpu_usage()).collect();
    let brand = if !cpus.is_empty() {
        cpus[0].brand().trim().to_string()
    } else {
        "Generic CPU".to_string()
    };

    let (user_percent, system_percent, iowait_percent) = read_proc_stat_cpu(overall_usage);

    let (load_1m, load_5m, load_15m) = get_load_average(sys);

    let core_count = cpus.len().max(1);
    let is_saturated = idle_percent < 10.0 || overall_usage > 90.0 || (load_1m > (core_count as f64 * 1.2));

    CpuMetrics {
        overall_usage,
        user_percent,
        system_percent,
        iowait_percent,
        idle_percent,
        core_usages,
        core_count,
        brand,
        load_avg_1m: load_1m,
        load_avg_5m: load_5m,
        load_avg_15m: load_15m,
        is_saturated,
    }
}

fn read_proc_stat_cpu(overall: f32) -> (f32, f32, f32) {
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        if let Some(first_line) = content.lines().next() {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 6 && parts[0] == "cpu" {
                let user: f64 = parts[1].parse().unwrap_or(0.0);
                let nice: f64 = parts[2].parse().unwrap_or(0.0);
                let system: f64 = parts[3].parse().unwrap_or(0.0);
                let idle: f64 = parts[4].parse().unwrap_or(0.0);
                let iowait: f64 = parts[5].parse().unwrap_or(0.0);

                let total = user + nice + system + idle + iowait;
                if total > 0.0 {
                    let u = ((user + nice) / total * 100.0) as f32;
                    let s = (system / total * 100.0) as f32;
                    let w = (iowait / total * 100.0) as f32;
                    return (u, s, w);
                }
            }
        }
    }
    // Fallback split proportional to overall usage
    (overall * 0.65, overall * 0.25, overall * 0.10)
}

fn get_load_average(_sys: &System) -> (f64, f64, f64) {
    let sys_load = System::load_average();
    if sys_load.one > 0.0 || sys_load.five > 0.0 {
        return (sys_load.one, sys_load.five, sys_load.fifteen);
    }

    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let l1 = parts[0].parse::<f64>().unwrap_or(0.0);
            let l5 = parts[1].parse::<f64>().unwrap_or(0.0);
            let l15 = parts[2].parse::<f64>().unwrap_or(0.0);
            return (l1, l5, l15);
        }
    }

    (0.0, 0.0, 0.0)
}
