use sysinfo::System;
use std::fs;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,       // MemTotal - MemAvailable
    pub available_bytes: u64,  // MemAvailable
    pub cached_bytes: u64,     // Buffers + Cached + SReclaimable
    pub usage_percent: f32,    // Raw used %
    pub memory_pressure: f32,  // True Pressure % based on MemAvailable
    pub is_under_pressure: bool,
}

pub fn collect_memory(sys: &System) -> MemoryMetrics {
    let mut total_bytes = sys.total_memory();
    let mut available_bytes = sys.available_memory();
    let mut cached_bytes = 0u64;

    // Parse /proc/meminfo for detailed Linux Cache & Buffer metrics
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;
        let mut cached = 0u64;
        let mut buffers = 0u64;
        let mut sreclaimable = 0u64;

        for line in content.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
                if let Ok(val) = val_str.parse::<u64>() {
                    let bytes = val * 1024;
                    match key {
                        "MemTotal:" => mem_total = bytes,
                        "MemAvailable:" => mem_available = bytes,
                        "Cached:" => cached = bytes,
                        "Buffers:" => buffers = bytes,
                        "SReclaimable:" => sreclaimable = bytes,
                        _ => {}
                    }
                }
            }
        }

        if mem_total > 0 {
            total_bytes = mem_total;
        }
        if mem_available > 0 {
            available_bytes = mem_available;
        }
        cached_bytes = cached + buffers + sreclaimable;
    }

    // Actual process memory used is total - available (since Cache is reclaimable)
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    let memory_pressure = if total_bytes > 0 {
        (used_bytes as f32 / total_bytes as f32) * 100.0
    } else {
        0.0
    };

    let usage_percent = if total_bytes > 0 {
        (sys.used_memory() as f32 / total_bytes as f32) * 100.0
    } else {
        0.0
    };

    // Genuine pressure occurs when true available RAM is under 12%
    let is_under_pressure = memory_pressure > 88.0 || (total_bytes > 0 && available_bytes < total_bytes / 8);

    MemoryMetrics {
        total_bytes,
        used_bytes,
        available_bytes,
        cached_bytes,
        usage_percent,
        memory_pressure,
        is_under_pressure,
    }
}
