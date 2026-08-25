use sysinfo::Disks;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiskMetrics {
    pub mount_point: String,
    pub name: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
}

pub fn collect_disks(disks: &Disks) -> Vec<DiskMetrics> {
    let mut list = Vec::new();
    for disk in disks {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        let mount = disk.mount_point().to_string_lossy().to_string();
        // Filter out tiny virtual mounts
        if total > 500 * 1024 * 1024 {
            list.push(DiskMetrics {
                mount_point: mount,
                name: disk.name().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                usage_percent,
            });
        }
    }

    // Sort by mount point length so / is first
    list.sort_by_key(|d| d.mount_point.len());
    list
}
