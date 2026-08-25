use crossterm::event::{KeyCode, KeyEvent};
use crate::app::state::{ActiveTab, AppState, ConfirmModal, PendingAction};
use crate::actions::{stop_process, toggle_service, run_cleanup_task};

pub fn handle_key_event(state: &mut AppState, key: KeyEvent) {
    // 0. If investigating offender modal is open
    if let Some(ref proc) = state.investigating_process.clone() {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('1') => {
                if proc.safety.is_system_critical {
                    state.set_notification(format!("Protected: Core system process '{}' cannot be terminated.", proc.name));
                } else {
                    let pid = proc.pid;
                    let name = proc.name.clone();
                    state.investigating_process = None;
                    state.confirm_modal = Some(ConfirmModal {
                        title: "STOP OFFENDER PROCESS".to_string(),
                        message: format!("Are you sure you want to stop runaway process '{}' (PID {})?", name, pid),
                        action: PendingAction::KillProcess(pid, name),
                    });
                }
            }
            KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Char('3') | KeyCode::Enter => {
                state.active_tab = ActiveTab::Processes;
                state.process_search = proc.name.clone();
                state.investigating_process = None;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                state.investigating_process = None;
                state.set_notification("Offender investigation closed.".to_string());
            }
            _ => {}
        }
        return;
    }

    // 0b. If "Why is my PC slow?" Diagnosis modal is open
    if state.show_diagnosis_modal {
        match key.code {
            KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Enter => {
                state.show_diagnosis_modal = false;
                // Auto-fix primary issue
                if let Some(runaway) = state.snapshot.processes.iter().find(|p| p.is_runaway).cloned() {
                    let pid = runaway.pid;
                    let name = runaway.name.clone();
                    state.confirm_modal = Some(ConfirmModal {
                        title: "FIX AUTOMATICALLY - STOP RUNAWAY PROCESS".to_string(),
                        message: format!("Stopping runaway process '{}' (PID {}) to restore CPU performance.", name, pid),
                        action: PendingAction::KillProcess(pid, name),
                    });
                } else if let Some(issue) = state.diagnosis.issues.first().cloned() {
                    if let Some(ref act) = issue.fixable_action {
                        if act.starts_with("kill:") {
                            if let Ok(pid) = act[5..].parse::<u32>() {
                                state.confirm_modal = Some(ConfirmModal {
                                    title: "FIX AUTOMATICALLY".to_string(),
                                    message: format!("Executing automatic remediation for PID {}.", pid),
                                    action: PendingAction::KillProcess(pid, format!("PID {}", pid)),
                                });
                            }
                        }
                    } else {
                        state.set_notification("System is currently optimized; no automatic fixes needed.".to_string());
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                state.show_diagnosis_modal = false;
                state.set_notification("Diagnosis dismissed.".to_string());
            }
            _ => {}
        }
        return;
    }

    // 1. If modal confirmation is open
    if let Some(ref modal) = state.confirm_modal.clone() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                let action = modal.action.clone();
                state.confirm_modal = None;
                execute_pending_action(state, action);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                state.confirm_modal = None;
                state.set_notification("Action cancelled".to_string());
            }
            _ => {}
        }
        return;
    }

    // 2. If search mode is active
    if state.is_searching {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                state.is_searching = false;
            }
            KeyCode::Backspace => {
                state.process_search.pop();
            }
            KeyCode::Char(c) => {
                state.process_search.push(c);
            }
            _ => {}
        }
        return;
    }

    // 3. Main Navigation & Hotkeys
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.should_quit = true;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            state.advanced_mode = !state.advanced_mode;
            let status = if state.advanced_mode { "Advanced telemetry mode enabled" } else { "Normal mode enabled" };
            state.set_notification(status.to_string());
        }
        KeyCode::Char('1') | KeyCode::Char('h') | KeyCode::Char('H') => {
            state.active_tab = ActiveTab::Dashboard;
        }
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char(' ') => {
            state.show_diagnosis_modal = true;
        }
        KeyCode::Char('2') | KeyCode::Char('p') | KeyCode::Char('P') => {
            state.active_tab = ActiveTab::Processes;
        }
        KeyCode::Char('3') | KeyCode::Char('s') | KeyCode::Char('S') => {
            state.active_tab = ActiveTab::Services;
        }
        KeyCode::Char('4') | KeyCode::Char('c') | KeyCode::Char('C') => {
            state.active_tab = ActiveTab::Cleanup;
        }
        KeyCode::Char('K') => {
            let dev_procs: Vec<(u32, String)> = state.snapshot.processes
                .iter()
                .filter(|p| p.is_dev_process)
                .map(|p| (p.pid, format!("{} (PID {})", p.name, p.pid)))
                .collect();

            if dev_procs.is_empty() {
                state.set_notification("No active developer processes found.".to_string());
            } else {
                let proc_list_str = dev_procs.iter().map(|(_, name)| format!(" * {}", name)).collect::<Vec<_>>().join("\n");
                state.confirm_modal = Some(ConfirmModal {
                    title: format!("STOP ALL DEVELOPER PROCESSES ({})", dev_procs.len()),
                    message: format!("Are you sure you want to stop all {} dev processes?\n\nProcesses to terminate:\n{}", dev_procs.len(), proc_list_str),
                    action: PendingAction::KillAllDev(dev_procs),
                });
            }
        }
        KeyCode::Char('/') | KeyCode::Char('f') => {
            if state.active_tab == ActiveTab::Processes {
                state.is_searching = true;
            }
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            if let Some(offender) = state.snapshot.processes.iter().find(|p| p.is_runaway).or_else(|| state.snapshot.processes.first()) {
                state.investigating_process = Some(offender.clone());
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_selection_down(state);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            move_selection_up(state);
        }
        KeyCode::Enter | KeyCode::Char('x') => {
            trigger_tab_action(state);
        }
        KeyCode::Esc => {
            if !state.process_search.is_empty() {
                state.process_search.clear();
            } else {
                state.active_tab = ActiveTab::Dashboard;
            }
        }
        _ => {}
    }
}

fn move_selection_down(state: &mut AppState) {
    match state.active_tab {
        ActiveTab::Dashboard => {}
        ActiveTab::Processes => {
            let count = state.get_filtered_processes().len();
            if count > 0 && state.selected_process_idx + 1 < count {
                state.selected_process_idx += 1;
            }
        }
        ActiveTab::Services => {
            let count = state.snapshot.services.len();
            if count > 0 && state.selected_service_idx + 1 < count {
                state.selected_service_idx += 1;
            }
        }
        ActiveTab::Cleanup => {
            let count = state.cleanup_tasks.len();
            if count > 0 && state.selected_cleanup_idx + 1 < count {
                state.selected_cleanup_idx += 1;
            }
        }
    }
}

fn move_selection_up(state: &mut AppState) {
    match state.active_tab {
        ActiveTab::Dashboard => {}
        ActiveTab::Processes => {
            if state.selected_process_idx > 0 {
                state.selected_process_idx -= 1;
            }
        }
        ActiveTab::Services => {
            if state.selected_service_idx > 0 {
                state.selected_service_idx -= 1;
            }
        }
        ActiveTab::Cleanup => {
            if state.selected_cleanup_idx > 0 {
                state.selected_cleanup_idx -= 1;
            }
        }
    }
}

fn trigger_tab_action(state: &mut AppState) {
    match state.active_tab {
        ActiveTab::Dashboard => {
            // Investigate runaway process or top CPU offender
            if let Some(offender) = state.snapshot.processes.iter().find(|p| p.is_runaway).or_else(|| state.snapshot.processes.first()) {
                state.investigating_process = Some(offender.clone());
            } else {
                state.set_notification("No CPU offenders detected.".to_string());
            }
        }
        ActiveTab::Processes => {
            let procs = state.get_filtered_processes();
            if let Some(proc) = procs.get(state.selected_process_idx) {
                state.confirm_modal = Some(ConfirmModal {
                    title: "STOP PROCESS".to_string(),
                    message: format!("Are you sure you want to stop process '{}' (PID {})?", proc.name, proc.pid),
                    action: PendingAction::KillProcess(proc.pid, proc.name.clone()),
                });
            }
        }
        ActiveTab::Services => {
            if let Some(svc) = state.snapshot.services.get(state.selected_service_idx) {
                let action_verb = if svc.active_state == "active" { "stop" } else { "start" };
                state.confirm_modal = Some(ConfirmModal {
                    title: format!("{} SERVICE", action_verb.to_uppercase()),
                    message: format!("Are you sure you want to {} service '{}'?", action_verb, svc.name),
                    action: PendingAction::ToggleService(svc.name.clone(), svc.active_state.clone()),
                });
            }
        }
        ActiveTab::Cleanup => {
            if let Some(task) = state.cleanup_tasks.get(state.selected_cleanup_idx) {
                state.confirm_modal = Some(ConfirmModal {
                    title: "RUN CLEANUP TASK".to_string(),
                    message: format!("Execute cleanup: '{}'?", task.name),
                    action: PendingAction::RunCleanup(task.id.to_string(), task.name.to_string()),
                });
            }
        }
    }
}

fn execute_pending_action(state: &mut AppState, action: PendingAction) {
    match action {
        PendingAction::KillProcess(pid, name) => {
            let initial_cpu = state.snapshot.cpu.overall_usage;
            match stop_process(pid, false) {
                Ok(_) => {
                    state.snapshot = state.collector.refresh();
                    let new_cpu = state.snapshot.cpu.overall_usage;
                    state.set_notification(format!("[OK] Process {} stopped. CPU usage dropped from {:.0}% -> {:.0}%", name, initial_cpu, new_cpu));
                }
                Err(e) => state.set_notification(format!("Error stopping {}: {}", name, e)),
            }
        }
        PendingAction::KillAllDev(procs) => {
            let mut stopped_count = 0;
            for (pid, _) in &procs {
                if stop_process(*pid, false).is_ok() {
                    stopped_count += 1;
                }
            }
            state.set_notification(format!("Stopped {} / {} developer processes.", stopped_count, procs.len()));
        }
        PendingAction::ToggleService(name, current_state) => {
            match toggle_service(&name, &current_state) {
                Ok(msg) => state.set_notification(msg),
                Err(e) => state.set_notification(format!("Error: {}", e)),
            }
        }
        PendingAction::RunCleanup(id, name) => {
            match run_cleanup_task(&id) {
                Ok(msg) => state.set_notification(msg),
                Err(e) => state.set_notification(format!("Error running {}: {}", name, e)),
            }
        }
    }
}
