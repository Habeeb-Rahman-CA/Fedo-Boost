use sysinfo::System;
use std::fs;
use std::time::Instant;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwapMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
    pub is_zram: bool,
    pub zram_device: String,
    pub swap_in_bytes_sec: u64,  // Reading speed from swap
    pub swap_out_bytes_sec: u64, // Writing speed to swap
    pub is_active_swapping: bool,
    pub status_explanation: String,
}

pub struct SwapCollector {
    prev_pswpin: u64,
    prev_pswpout: u64,
    last_tick: Instant,
}

impl SwapCollector {
    pub fn new() -> Self {
        let (in_pages, out_pages) = Self::read_proc_vmstat();
        Self {
            prev_pswpin: in_pages,
            prev_pswpout: out_pages,
            last_tick: Instant::now(),
        }
    }

    fn read_proc_vmstat() -> (u64, u64) {
        let mut pswpin = 0u64;
        let mut pswpout = 0u64;

        if let Ok(content) = fs::read_to_string("/proc/vmstat") {
            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
                    if key == "pswpin" {
                        pswpin = val_str.parse::<u64>().unwrap_or(0);
                    } else if key == "pswpout" {
                        pswpout = val_str.parse::<u64>().unwrap_or(0);
                    }
                }
            }
        }
        (pswpin, pswpout)
    }

    pub fn collect(&mut self, sys: &System) -> SwapMetrics {
        let total_bytes = sys.total_swap();
        let used_bytes = sys.used_swap();
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        let mut is_zram = false;
        let mut zram_device = String::from("Swap");

        if let Ok(swaps_content) = fs::read_to_string("/proc/swaps") {
            for line in swaps_content.lines() {
                if line.contains("zram") {
                    is_zram = true;
                    if let Some(dev) = line.split_whitespace().next() {
                        zram_device = dev.to_string();
                    }
                    break;
                }
            }
        }

        // Calculate Swap In (Reading) / Swap Out (Writing) rates via vmstat
        let (curr_in_pages, curr_out_pages) = Self::read_proc_vmstat();
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs_f64().max(0.1);
        self.last_tick = now;

        // Page size in Linux is 4096 bytes (4 KB)
        let page_size = 4096u64;
        let swap_in_bytes_sec = if curr_in_pages >= self.prev_pswpin {
            (((curr_in_pages - self.prev_pswpin) * page_size) as f64 / elapsed) as u64
        } else { 0 };

        let swap_out_bytes_sec = if curr_out_pages >= self.prev_pswpout {
            (((curr_out_pages - self.prev_pswpout) * page_size) as f64 / elapsed) as u64
        } else { 0 };

        self.prev_pswpin = curr_in_pages;
        self.prev_pswpout = curr_out_pages;

        // Active swapping detected if page thrashing > 50 KB/s
        let is_active_swapping = swap_in_bytes_sec > 50 * 1024 || swap_out_bytes_sec > 50 * 1024;

        let status_explanation = if is_active_swapping {
            "[!] Active swapping detected".to_string()
        } else if is_zram {
            "[OK] ZRAM is currently healthy.".to_string()
        } else {
            "[OK] Swap is idle.".to_string()
        };

        SwapMetrics {
            total_bytes,
            used_bytes,
            usage_percent,
            is_zram,
            zram_device,
            swap_in_bytes_sec,
            swap_out_bytes_sec,
            is_active_swapping,
            status_explanation,
        }
    }
}

#[allow(dead_code)]
pub fn format_swap_rate(bytes_per_sec: u64) -> String {
    let mb_per_sec = bytes_per_sec as f64 / (1024.0 * 1024.0);
    format!("{:.1} MB/s", mb_per_sec)
}
