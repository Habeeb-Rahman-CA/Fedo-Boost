pub mod dashboard;
pub mod processes;
pub mod services;
pub mod cleanup;
pub mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};

use crate::app::state::{ActiveTab, AppState};

pub fn render_app(frame: &mut Frame, state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab navigation bar
            Constraint::Min(10),   // Active view content
            Constraint::Length(if state.notification.is_some() { 3 } else { 0 }), // Toast notification if any
        ])
        .split(frame.area());

    render_tabs(frame, main_layout[0], state);

    match state.active_tab {
        ActiveTab::Dashboard => dashboard::render_dashboard(frame, main_layout[1], state),
        ActiveTab::Processes => processes::render_processes(frame, main_layout[1], state),
        ActiveTab::Services => services::render_services(frame, main_layout[1], state),
        ActiveTab::Cleanup => cleanup::render_cleanup(frame, main_layout[1], state),
    }

    if let Some(ref note) = state.notification {
        let p = widgets::render_notification(note);
        frame.render_widget(p, main_layout[2]);
    }

    if state.show_diagnosis_modal {
        render_diagnosis_modal(frame, state);
    } else if let Some(ref proc) = state.investigating_process {
        render_offender_modal(frame, proc);
    } else if let Some(ref modal) = state.confirm_modal {
        render_modal(frame, modal.title.as_str(), modal.message.as_str());
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let titles = vec![
        " [1] Home ",
        " [2] Activity ",
        " [3] Services ",
        " [4] Cleanup ",
    ];

    let tab_index = match state.active_tab {
        ActiveTab::Dashboard => 0,
        ActiveTab::Processes => 1,
        ActiveTab::Services => 2,
        ActiveTab::Cleanup => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Rgb(60, 110, 180))))
        .select(tab_index)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    frame.render_widget(tabs, area);
}

fn render_modal(frame: &mut Frame, title: &str, message: &str) {
    let msg_lines: Vec<Line> = message.lines().map(|l| Line::from(Span::styled(l, Style::default().fg(Color::White)))).collect();
    let area = centered_rect(65, (msg_lines.len() as u16 + 8).min(80), frame.area());

    frame.render_widget(Clear, area);

    let mut text = vec![Line::from("")];
    text.extend(msg_lines);
    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled(" [y] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm  ", Style::default().fg(Color::White)),
        Span::styled("[n] / [Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(Color::White)),
    ]));

    let modal_block = Paragraph::new(text)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", title), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(modal_block, area);
}

fn render_offender_modal(frame: &mut Frame, proc: &crate::system::ProcessInfo) {
    let area = centered_rect(70, 65, frame.area());

    frame.render_widget(Clear, area);

    let mem_mb = proc.memory_bytes / 1024 / 1024;

    let mut text = vec![
        Line::from(Span::styled("  [!] PERFORMANCE ISSUE", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Process:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} (PID {})", proc.name, proc.pid), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  CPU        ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.0}%", proc.cpu_usage), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  RAM        ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} MB", mem_mb), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Running    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(&proc.run_time_formatted, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Command    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(if proc.cmd.len() > 40 { format!("{}...", &proc.cmd[..37]) } else { proc.cmd.clone() }, Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  SAFETY & RISK ASSESSMENT", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  Risk:      ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(
                &proc.safety.risk_label,
                if proc.safety.is_system_critical {
                    Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                },
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Reason:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
    ];

    for reason in &proc.safety.reasons {
        text.push(Line::from(Span::styled(format!("  {}", reason), Style::default().fg(Color::White))));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("  Recommended action: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            &proc.safety.recommended_action,
            if proc.safety.is_system_critical {
                Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            },
        ),
    ]));

    text.push(Line::from(""));
    text.push(Line::from(Span::styled("  EXPLANATION", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    text.push(Line::from(Span::styled(format!("  {}", proc.explanation), Style::default().fg(Color::White))));
    text.push(Line::from(""));
    
    if proc.safety.is_system_critical {
        text.push(Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Dismiss (System Process Protected)", Style::default().fg(Color::White)),
        ]));
    } else {
        text.push(Line::from(vec![
            Span::styled("  [s] / [1] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("Stop Process   ", Style::default().fg(Color::White)),
            Span::styled("[i] / [3] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Investigate   ", Style::default().fg(Color::White)),
            Span::styled("[Esc] ", Style::default().fg(Color::Gray)),
            Span::styled("Ignore", Style::default().fg(Color::White)),
        ]));
    }

    let border_color = if proc.safety.is_system_critical { Color::Yellow } else { Color::Red };

    let block = Paragraph::new(text)
        .block(
            Block::default()
                .title(Span::styled(" FEDORA BOOST - PERFORMANCE INVESTIGATOR ", Style::default().fg(border_color).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

    frame.render_widget(block, area);
}

fn render_diagnosis_modal(frame: &mut Frame, state: &AppState) {
    let area = centered_rect(65, 75, frame.area());
    frame.render_widget(Clear, area);

    let mut text = vec![Line::from("")];

    // CPU Status
    let cpu_badge = if state.diagnosis.cpu_healthy { "[OK] CPU" } else { "[X] CPU" };
    let cpu_style = if state.diagnosis.cpu_healthy { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) };
    text.push(Line::from(Span::styled(format!("  {}", cpu_badge), cpu_style)));
    text.push(Line::from(""));
    
    if let Some(runaway) = state.snapshot.processes.iter().find(|p| p.is_runaway) {
        text.push(Line::from(Span::styled(format!("  {} is consuming {:.0}% CPU.", runaway.name, runaway.cpu_usage), Style::default().fg(Color::White))));
    } else {
        text.push(Line::from(Span::styled(format!("  Overall CPU utilization is at {:.1}%.", state.snapshot.cpu.overall_usage), Style::default().fg(Color::White))));
    }
    text.push(Line::from(""));

    // RAM Status
    let ram_badge = if state.diagnosis.memory_healthy { "[OK] RAM" } else { "[X] RAM" };
    let ram_style = if state.diagnosis.memory_healthy { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) };
    text.push(Line::from(Span::styled(format!("  {}", ram_badge), ram_style)));
    text.push(Line::from(""));
    let avail_gb = state.snapshot.memory.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    text.push(Line::from(Span::styled(format!("  {:.1} GB available.", avail_gb), Style::default().fg(Color::White))));
    text.push(Line::from(""));

    // STORAGE Status
    let disk_badge = if state.diagnosis.storage_healthy { "[OK] STORAGE" } else { "[X] STORAGE" };
    let disk_style = if state.diagnosis.storage_healthy { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) };
    text.push(Line::from(Span::styled(format!("  {}", disk_badge), disk_style)));
    text.push(Line::from(""));
    let disk_avail_gb = state.snapshot.disks.first().map(|d| d.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0).unwrap_or(0.0);
    text.push(Line::from(Span::styled(format!("  {:.0} GB available.", disk_avail_gb), Style::default().fg(Color::White))));
    text.push(Line::from(""));

    // ZRAM Status
    let zram_badge = if state.diagnosis.swap_healthy { "[OK] ZRAM" } else { "[X] ZRAM" };
    let zram_style = if state.diagnosis.swap_healthy { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) };
    text.push(Line::from(Span::styled(format!("  {}", zram_badge), zram_style)));
    text.push(Line::from(""));
    text.push(Line::from(Span::styled(format!("  {}", state.snapshot.swap.status_explanation), Style::default().fg(Color::White))));
    text.push(Line::from(""));

    text.push(Line::from(Span::styled("  RECOMMENDATION", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    text.push(Line::from(""));

    if let Some(issue) = state.diagnosis.issues.first() {
        text.push(Line::from(Span::styled(format!("  {}", issue.recommendation), Style::default().fg(Color::White))));
    } else {
        text.push(Line::from(Span::styled("  No critical performance bottlenecks detected.", Style::default().fg(Color::Green))));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("  [ Fix Automatically ] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Press [f] or [Enter]   ", Style::default().fg(Color::White)),
        Span::styled("[Esc] ", Style::default().fg(Color::Gray)),
        Span::styled("Dismiss", Style::default().fg(Color::White)),
    ]));

    let block = Paragraph::new(text)
        .block(
            Block::default()
                .title(Span::styled(" DIAGNOSIS ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(block, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
