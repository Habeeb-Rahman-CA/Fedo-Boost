pub mod cpu;
pub mod memory;
pub mod swap;
pub mod disk;
pub mod processes;
pub mod services;
pub mod network;

pub mod network_ports;
pub mod browser;

pub use cpu::CpuMetrics;
pub use memory::MemoryMetrics;
pub use swap::SwapMetrics;
pub use disk::DiskMetrics;
pub use processes::{ProcessInfo, ProcessCollector};
pub use services::{ServiceInfo, ServiceCollector};
pub use network::NetworkMetrics;
pub use network_ports::{ListeningPortInfo, collect_listening_ports};
pub use browser::{BrowserSummary, analyze_browser_processes};

use sysinfo::{Disks, Networks, System};

#[allow(dead_code)]
pub struct SystemSnapshot {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub swap: SwapMetrics,
    pub disks: Vec<DiskMetrics>,
    pub processes: Vec<ProcessInfo>,
    pub services: Vec<ServiceInfo>,
    pub network: NetworkMetrics,
    pub listening_ports: Vec<ListeningPortInfo>,
    pub browser: Option<BrowserSummary>,
}

pub struct SystemCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    process_collector: ProcessCollector,
    service_collector: ServiceCollector,
    swap_collector: swap::SwapCollector,
    prev_net_rx: u64,
    prev_net_tx: u64,
    last_tick: std::time::Instant,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        
        let (rx, tx) = Self::calc_total_net(&networks);

        Self {
            sys,
            disks,
            networks,
            process_collector: ProcessCollector::new(),
            service_collector: ServiceCollector::new(),
            swap_collector: swap::SwapCollector::new(),
            prev_net_rx: rx,
            prev_net_tx: tx,
            last_tick: std::time::Instant::now(),
        }
    }

    fn calc_total_net(networks: &Networks) -> (u64, u64) {
        let mut rx = 0;
        let mut tx = 0;
        for (_, net) in networks {
            rx += net.received();
            tx += net.transmitted();
        }
        (rx, tx)
    }

    pub fn refresh(&mut self) -> SystemSnapshot {
        self.sys.refresh_all();
        self.disks = Disks::new_with_refreshed_list();
        self.networks = Networks::new_with_refreshed_list();

        let cpu = cpu::collect_cpu(&self.sys);
        let memory = memory::collect_memory(&self.sys);
        let swap = self.swap_collector.collect(&self.sys);
        let disks = disk::collect_disks(&self.disks);
        let processes = self.process_collector.collect(&self.sys);
        let services = self.service_collector.collect(&processes);

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs_f64().max(0.1);
        self.last_tick = now;

        let (curr_rx, curr_tx) = Self::calc_total_net(&self.networks);
        let rx_rate = if curr_rx >= self.prev_net_rx {
            ((curr_rx - self.prev_net_rx) as f64 / elapsed) as u64
        } else { 0 };

        let tx_rate = if curr_tx >= self.prev_net_tx {
            ((curr_tx - self.prev_net_tx) as f64 / elapsed) as u64
        } else { 0 };

        self.prev_net_rx = curr_rx;
        self.prev_net_tx = curr_tx;

        let network = NetworkMetrics {
            rx_bytes_per_sec: rx_rate,
            tx_bytes_per_sec: tx_rate,
        };

        let listening_ports = collect_listening_ports(&processes);
        let browser = analyze_browser_processes(&processes);

        SystemSnapshot {
            cpu,
            memory,
            swap,
            disks,
            processes,
            services,
            network,
            listening_ports,
            browser,
        }
    }
}
