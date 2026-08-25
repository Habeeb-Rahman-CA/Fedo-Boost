#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[allow(dead_code)]
impl NetworkMetrics {
    pub fn format_rx(&self) -> String {
        format_rate(self.rx_bytes_per_sec)
    }

    pub fn format_tx(&self) -> String {
        format_rate(self.tx_bytes_per_sec)
    }
}

fn format_rate(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B/s", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB/s", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB/s", bytes as f64 / (1024.0 * 1024.0))
    }
}
