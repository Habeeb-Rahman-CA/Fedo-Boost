use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Table, Row, Cell},
    Frame,
};

use crate::app::state::AppState;
use crate::ui::widgets::build_block;

pub fn render_cleanup(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Storage Cleanup Table
            Constraint::Length(4), // Educational Explanation Banner
            Constraint::Length(3), // Action Bar
        ])
        .split(area);

    render_cleanup_table(frame, chunks[0], state);
    render_explanation_banner(frame, chunks[1]);
    render_action_bar(frame, chunks[2]);
}

fn render_cleanup_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let header_cells = ["TARGET", "RECLAIMABLE STORAGE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let tasks = &state.cleanup_tasks;

    let rows = tasks.iter().enumerate().map(|(idx, task)| {
        let is_selected = idx == state.selected_cleanup_idx;

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(40, 60, 90)).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::from(task.name).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Cell::from(task.formatted_size.as_str()).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]).style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(25),
            Constraint::Length(25),
        ],
    )
    .header(header)
    .block(build_block(" Cleanup ", true));

    frame.render_widget(table, area);
}

fn render_explanation_banner(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled("Cleaning these items mainly frees storage. It usually won't improve CPU/RAM performance.", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("Fedora manages RAM cache automatically for fast system responsiveness.", Style::default().fg(Color::DarkGray))),
    ];

    let p = Paragraph::new(text).block(build_block(" STORAGE vs PERFORMANCE ", false));
    frame.render_widget(p, area);
}

fn render_action_bar(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" [ Review ] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Press [Enter] or [c] to Clean Selected Target  ", Style::default().fg(Color::White)),
        Span::styled("[Up/Down] ", Style::default().fg(Color::Cyan)),
        Span::styled("Navigate Targets", Style::default().fg(Color::White)),
    ]);

    let p = Paragraph::new(text)
        .block(build_block(" ACTIONS ", false));
    frame.render_widget(p, area);
}
