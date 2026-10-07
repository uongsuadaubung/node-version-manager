use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::NvmApp;
use super::utils::centered_rect;

pub fn draw_confirm_delete_modal(f: &mut Frame, app: &NvmApp, version: &str) {
    let area = centered_rect(50, 25, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", app.i18n.t("ui.confirm_delete_title")))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Red).bold());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                app.i18n.t("ui.confirm_delete_msg").replace("{}", version),
                Style::default().fg(Color::White).bold(),
            ),
        ]),
        Line::styled(app.i18n.t("ui.confirm_delete_note"), Style::default().fg(Color::DarkGray)),
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!("  {}  ", app.i18n.t("ui.confirm_yes")), Style::default().bg(Color::Red).fg(Color::White).bold()),
            Span::raw("    "),
            Span::styled(format!("  {}  ", app.i18n.t("ui.confirm_no")), Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
        ]),
    ];

    let p = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(p, inner);
}

