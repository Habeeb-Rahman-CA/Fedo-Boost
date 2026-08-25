use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Table, Row, Cell},
    Frame,
};

use crate::app::state::AppState;
use crate::ui::widgets::build_block;

pub fn render_services(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Simple Services Table
            Constraint::Length(3), // Manage Action Bar
        ])
        .split(area);

    render_simple_services_table(frame, chunks[0], state);
    render_manage_action_bar(frame, chunks[1]);
}

fn render_simple_services_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let header_cells = ["SERVICE", "STATE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let services = &state.snapshot.services;

    let rows = services.iter().enumerate().map(|(idx, svc)| {
        let is_selected = idx == state.selected_service_idx;

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(40, 60, 90)).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let (state_str, state_style) = if svc.active_state == "active" {
            ("Running", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            ("Stopped", Style::default().fg(Color::DarkGray))
        };

        Row::new(vec![
            Cell::from(svc.display_name.as_str()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Cell::from(state_str).style(state_style),
        ]).style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(25),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(build_block(" Services ", true));

    frame.render_widget(table, area);
}

fn render_manage_action_bar(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" [ Manage ] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Press [s] or [Enter] to Start / Stop Service  ", Style::default().fg(Color::White)),
        Span::styled("[Up/Down] ", Style::default().fg(Color::Cyan)),
        Span::styled("Navigate", Style::default().fg(Color::White)),
    ]);

    let p = Paragraph::new(text)
        .block(build_block(" ACTIONS ", false));
    frame.render_widget(p, area);
}
