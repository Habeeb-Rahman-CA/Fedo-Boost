use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::AppState;
use crate::ui::widgets::build_block;

pub fn render_dashboard(frame: &mut Frame, area: Rect, state: &AppState) {
    let snap = &state.snapshot;

    let runaway_proc = snap.processes.iter().find(|p| p.is_runaway);
    let dev_server_count = snap.processes.iter().filter(|p| p.is_dev_process).count();
    let active_svc_count = snap.services.iter().filter(|s| s.active_state == "active").count();

    let mut lines = Vec::new();

    // Top Header: boost                                   Fedora
    lines.push(Line::from(vec![
        Span::styled("boost", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("                                              Fedora", Style::default().fg(Color::Rgb(60, 110, 180)).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Hero Status Message
    if let Some(proc) = runaway_proc {
        lines.push(Line::from(Span::styled("  Your system needs attention", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));

        // Attention Box - pure ASCII box
        lines.push(Line::from(Span::styled("  +------------------------------------------------+", Style::default().fg(Color::Red))));
        lines.push(Line::from(vec![
            Span::styled("  |  ", Style::default().fg(Color::Red)),
            Span::styled("High CPU usage", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("                                |", Style::default().fg(Color::Red)),
        ]));
        lines.push(Line::from(Span::styled("  |                                                |", Style::default().fg(Color::Red))));
        lines.push(Line::from(vec![
            Span::styled("  |  ", Style::default().fg(Color::Red)),
            Span::styled(&proc.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("                                              |", Style::default().fg(Color::Red)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  |  Using ", Style::default().fg(Color::Red)),
            Span::styled(format!("{:.0}% CPU", proc.cpu_usage), Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
            Span::styled("                                |", Style::default().fg(Color::Red)),
        ]));
        lines.push(Line::from(Span::styled("  |                                                |", Style::default().fg(Color::Red))));
        lines.push(Line::from(vec![
            Span::styled("  |  Running for ", Style::default().fg(Color::Red)),
            Span::styled(&proc.run_time_formatted, Style::default().fg(Color::Yellow)),
            Span::styled("                                 |", Style::default().fg(Color::Red)),
        ]));
        lines.push(Line::from(Span::styled("  |                                                |", Style::default().fg(Color::Red))));
        lines.push(Line::from(vec![
            Span::styled("  |                 ", Style::default().fg(Color::Red)),
            Span::styled("[ Fix ] (Press Space / Enter)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("           |", Style::default().fg(Color::Red)),
        ]));
        lines.push(Line::from(Span::styled("  +------------------------------------------------+", Style::default().fg(Color::Red))));
    } else {
        lines.push(Line::from(Span::styled("  Your system is healthy", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(Span::styled("  Everything looks good.", Style::default().fg(Color::DarkGray))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  --------------------------------------------------", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(""));

    let ram_used_gb = snap.memory.used_bytes as f64 / 1e9;
    let ram_tot_gb = snap.memory.total_bytes as f64 / 1e9;

    if !state.advanced_mode {
        // Normal User Mode: Simple & Clean
        let cpu_status = if snap.cpu.overall_usage > 75.0 { "High load" } else { "Normal" };
        let ram_status = if snap.memory.is_under_pressure { "Under pressure" } else { "Normal" };

        lines.push(Line::from(vec![
            Span::styled("  CPU                         ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Memory", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<27.0}% ", snap.cpu.overall_usage), Style::default().fg(Color::White)),
            Span::styled(format!("{:.1} / {:.1} GB", ram_used_gb, ram_tot_gb), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<27} ", cpu_status), Style::default().fg(if snap.cpu.overall_usage > 75.0 { Color::Red } else { Color::Green })),
            Span::styled(ram_status, Style::default().fg(if snap.memory.is_under_pressure { Color::Yellow } else { Color::Green })),
        ]));
    } else {
        // Advanced User Mode: Deep Telemetry Breakdown
        lines.push(Line::from(Span::styled("  CPU (ADVANCED TELEMETRY)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(Span::styled(format!("    user:       {:>5.1}%", snap.cpu.user_percent), Style::default().fg(Color::White))));
        lines.push(Line::from(Span::styled(format!("    system:     {:>5.1}%", snap.cpu.system_percent), Style::default().fg(Color::White))));
        lines.push(Line::from(Span::styled(format!("    iowait:     {:>5.1}%", snap.cpu.iowait_percent), Style::default().fg(Color::White))));
        lines.push(Line::from(Span::styled(format!("    idle:       {:>5.1}%", snap.cpu.idle_percent), Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Load average", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(Span::styled(format!("    1m:  {:.1}", snap.cpu.load_avg_1m), Style::default().fg(Color::White))));
        lines.push(Line::from(Span::styled(format!("    5m:  {:.1}", snap.cpu.load_avg_5m), Style::default().fg(Color::White))));
        lines.push(Line::from(Span::styled(format!("   15m:  {:.1}", snap.cpu.load_avg_15m), Style::default().fg(Color::White))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  --------------------------------------------------", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(""));

    // Background Activity
    lines.push(Line::from(Span::styled("  Background activity", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(format!("  {} development servers", dev_server_count), Style::default().fg(Color::White))));
    lines.push(Line::from(Span::styled(format!("  {} services running", active_svc_count), Style::default().fg(Color::White))));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [ Review ] (Press [2] or [d])  | ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(
            if state.advanced_mode { "[a] Toggle Normal Mode" } else { "[a] Toggle Advanced Telemetry" },
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  --------------------------------------------------", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Last checked: Just now", Style::default().fg(Color::DarkGray))));

    let p = Paragraph::new(lines)
        .block(build_block(" HOME ", true));
    frame.render_widget(p, area);
}
