use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Table, Row, Cell},
    Frame,
};

use crate::app::state::AppState;
use crate::ui::widgets::build_block;

pub fn render_processes(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search & Filter bar
            Constraint::Min(10),   // Activity Table
            Constraint::Length(3), // Action bar: Stop, Details, Ignore
        ])
        .split(area);

    render_search_bar(frame, chunks[0], state);
    render_activity_table(frame, chunks[1], state);
    render_action_bar(frame, chunks[2]);
}

fn render_search_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let filter_text = if state.process_search.is_empty() {
        Span::styled("[ Search processes... ]  Press [/]", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(format!("Search: {}", state.process_search), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    };

    let p = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        filter_text,
    ]))
    .block(build_block(" SEARCH & FILTER ", state.is_searching));

    frame.render_widget(p, area);
}

fn render_activity_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let header_cells = ["RESOURCE", "PROCESS NAME", "MEMORY", "STATUS"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let filtered_procs = state.get_filtered_processes();

    let rows = filtered_procs.iter().enumerate().map(|(idx, proc)| {
        let is_selected = idx == state.selected_process_idx;

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(40, 60, 90)).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let mem_mb = proc.memory_bytes / 1024 / 1024;
        let cpu_str = format!("{:>4.0}%", proc.cpu_usage);

        let (status_str, status_style) = if proc.is_runaway {
            ("[!] ATTENTION REQUIRED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        } else if proc.is_dev_process {
            ("DEVELOPMENT", Style::default().fg(Color::Yellow))
        } else {
            ("NORMAL", Style::default().fg(Color::DarkGray))
        };

        Row::new(vec![
            Cell::from(cpu_str).style(if proc.is_runaway { Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Yellow) }),
            Cell::from(proc.name.as_str()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{} MB", mem_mb)).style(Style::default().fg(Color::White)),
            Cell::from(status_str).style(status_style),
        ]).style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Min(25),
            Constraint::Length(14),
            Constraint::Length(22),
        ],
    )
    .header(header)
    .block(build_block(" Activity ", true));

    frame.render_widget(table, area);
}

fn render_action_bar(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" [Stop] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("Press [k] or [x]  ", Style::default().fg(Color::White)),
        Span::styled("[Details] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Press [i] or [Enter]  ", Style::default().fg(Color::White)),
        Span::styled("[Ignore] ", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled("Press [Esc]  ", Style::default().fg(Color::White)),
        Span::styled("[Up/Down] ", Style::default().fg(Color::Yellow)),
        Span::styled("Navigate", Style::default().fg(Color::White)),
    ]);

    let p = Paragraph::new(text)
        .block(build_block(" ACTIONS ", false));
    frame.render_widget(p, area);
}
