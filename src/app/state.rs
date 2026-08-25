use crate::system::{SystemCollector, SystemSnapshot, ProcessInfo};
use crate::diagnosis::{DiagnosisReport, SystemAnalyzer};
use crate::actions::cleanup::{get_cleanup_tasks, CleanupTask};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    Processes,
    Services,
    Cleanup,
}

#[derive(Debug, Clone)]
pub struct ConfirmModal {
    pub title: String,
    pub message: String,
    pub action: PendingAction,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    KillProcess(u32, String),
    KillAllDev(Vec<(u32, String)>),
    ToggleService(String, String),
    RunCleanup(String, String),
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub collector: SystemCollector,
    pub snapshot: SystemSnapshot,
    pub diagnosis: DiagnosisReport,
    pub cleanup_tasks: Vec<CleanupTask>,

    pub selected_process_idx: usize,
    pub process_search: String,
    pub is_searching: bool,

    pub selected_service_idx: usize,
    pub selected_cleanup_idx: usize,

    pub notification: Option<String>,
    pub notification_ticks: usize,

    pub confirm_modal: Option<ConfirmModal>,
    pub investigating_process: Option<ProcessInfo>,
    pub show_diagnosis_modal: bool,
    pub advanced_mode: bool,
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        let mut collector = SystemCollector::new();
        let snapshot = collector.refresh();
        let diagnosis = SystemAnalyzer::analyze(&snapshot);
        let cleanup_tasks = get_cleanup_tasks();

        Self {
            active_tab: ActiveTab::Dashboard,
            collector,
            snapshot,
            diagnosis,
            cleanup_tasks,

            selected_process_idx: 0,
            process_search: String::new(),
            is_searching: false,

            selected_service_idx: 0,
            selected_cleanup_idx: 0,

            notification: None,
            notification_ticks: 0,

            confirm_modal: None,
            investigating_process: None,
            show_diagnosis_modal: false,
            advanced_mode: false,
            should_quit: false,
        }
    }

    pub fn tick(&mut self) {
        self.snapshot = self.collector.refresh();
        self.diagnosis = SystemAnalyzer::analyze(&self.snapshot);

        if self.notification.is_some() {
            if self.notification_ticks > 0 {
                self.notification_ticks -= 1;
            } else {
                self.notification = None;
            }
        }
    }

    pub fn set_notification(&mut self, msg: String) {
        self.notification = Some(msg);
        self.notification_ticks = 4; // Display for ~4 seconds/ticks
    }

    pub fn get_filtered_processes(&self) -> Vec<&ProcessInfo> {
        if self.process_search.is_empty() {
            self.snapshot.processes.iter().collect()
        } else {
            let q = self.process_search.to_lowercase();
            self.snapshot
                .processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&q)
                        || p.pid.to_string().contains(&q)
                        || p.cmd.to_lowercase().contains(&q)
                })
                .collect()
        }
    }
}
