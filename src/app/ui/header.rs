use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::NvmApp;

pub fn draw_header(f: &mut Frame, app: &NvmApp, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    let not_selected = app.i18n.t("ui.not_selected");
    let active_ver = app
        .config
        .current_version
        .as_deref()
        .unwrap_or(&not_selected);

    let (status_icon, status_style) = if app.config.current_version.is_some() {
        ("●", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))
    } else {
        ("○", Style::default().fg(Color::DarkGray))
    };

    let title_line = Line::from(vec![
        Span::styled(" ⚡ NVM-RS ", Style::default().bg(Color::Cyan).fg(Color::Black).bold()),
        Span::raw("  "),
        Span::styled(format!("{} ", status_icon), status_style),
        Span::styled(
            app.i18n.t("ui.current_version").replace("{}", active_ver),
            Style::default().fg(Color::White).bold(),
        ),
    ]);

    let left_para = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(left_para, header_chunks[0]);

    let base_dir_str = app.config.base_dir.to_string_lossy();
    let lang_display = app.i18n.t("ui.lang_badge");

    let right_line = Line::from(vec![
        Span::styled("📁 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            app.i18n.t("ui.saved_at").replace("{}", &base_dir_str),
            Style::default().fg(Color::Gray),
        ),
        Span::raw("  "),
        Span::styled(format!("[L: {}]", lang_display), Style::default().fg(Color::Yellow).bold()),
    ]);

    let right_para = Paragraph::new(right_line)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(right_para, header_chunks[1]);
}
