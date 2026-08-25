use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Build subtle, non-intrusive container block
pub fn build_subtle_block(title: &str, is_active: bool) -> Block<'static> {
    let border_style = if is_active {
        Style::default().fg(Color::Rgb(80, 90, 105)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(45, 50, 60))
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
}

pub fn build_block(title: &str, is_active: bool) -> Block<'static> {
    build_subtle_block(title, is_active)
}

/// Standard ASCII State Indicators (Pure ASCII compliant)
#[allow(dead_code)]
pub fn render_status_badge<'a>(state: &str) -> Span<'a> {
    match state {
        "healthy" | "ok" | "running" => Span::styled("[OK] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        "attention" | "warning" => Span::styled("[!] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        "critical" | "runaway" | "failed" => Span::styled("[X] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        _ => Span::styled("[-] ", Style::default().fg(Color::DarkGray)),
    }
}

pub fn render_notification<'a>(message: &'a str) -> Paragraph<'a> {
    Paragraph::new(Line::from(vec![
        Span::styled(" INFO ", Style::default().fg(Color::Black).bg(Color::Rgb(180, 190, 200)).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(message, Style::default().fg(Color::White)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Rgb(80, 90, 105))))
}
