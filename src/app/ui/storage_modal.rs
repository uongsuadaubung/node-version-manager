use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::NvmApp;

pub fn draw_change_storage_modal(f: &mut Frame, app: &NvmApp) {
    let screen = f.area();
    let modal_width = (screen.width * 75 / 100).max(50).min(screen.width.saturating_sub(2));
    let modal_height = 11.min(screen.height.saturating_sub(2));

    let area = Rect {
        x: (screen.width.saturating_sub(modal_width)) / 2,
        y: (screen.height.saturating_sub(modal_height)) / 2,
        width: modal_width,
        height: modal_height,
    };
    f.render_widget(Clear, area);

    let title = app.i18n.t("ui.storage_modal_title");

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow).bold());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Current path
            Constraint::Length(3), // Input box
            Constraint::Length(2), // Hint note
            Constraint::Length(1), // Buttons
        ])
        .split(inner);

    let cur_str = app.config.base_dir.to_string_lossy();
    let current_line = Line::from(vec![
        Span::styled(app.i18n.t("ui.storage_current_path"), Style::default().fg(Color::DarkGray)),
        Span::styled(cur_str, Style::default().fg(Color::Cyan).bold()),
    ]);
    f.render_widget(Paragraph::new(current_line), chunks[0]);

    let input_block = Block::default()
        .title(app.i18n.t("ui.storage_input_title"))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightGreen));

    let display_input = if app.storage_input.is_empty() {
        Line::from(vec![
            Span::styled(app.i18n.t("ui.storage_input_placeholder"), Style::default().fg(Color::DarkGray)),
            Span::styled("█", Style::default().fg(Color::LightGreen)),
        ])
    } else {
        Line::from(vec![
            Span::styled(&app.storage_input, Style::default().fg(Color::White).bold()),
            Span::styled("█", Style::default().fg(Color::LightGreen)),
        ])
    };
    let input_para = Paragraph::new(display_input).block(input_block);
    f.render_widget(input_para, chunks[1]);

    let hint_text = app.i18n.t("ui.storage_hint");
    f.render_widget(Paragraph::new(hint_text).style(Style::default().fg(Color::Yellow)), chunks[2]);

    let buttons_line = Line::from(vec![
        Span::styled(app.i18n.t("ui.storage_save_btn"), Style::default().bg(Color::Green).fg(Color::Black).bold()),
        Span::raw("    "),
        Span::styled(app.i18n.t("ui.storage_cancel_btn"), Style::default().bg(Color::DarkGray).fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(buttons_line).alignment(Alignment::Center), chunks[3]);
}

