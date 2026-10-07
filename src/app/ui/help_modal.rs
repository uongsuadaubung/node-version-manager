use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::NvmApp;
use super::utils::centered_rect;

pub fn draw_help_modal(f: &mut Frame, app: &NvmApp) {
    let area = centered_rect(65, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", app.i18n.t("ui.help_title")))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan).bold());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let shortcuts: Vec<(&str, &str)> = vec![
        ("↑ / k", app.i18n.t("ui.help_nav")),
        ("↓ / j", app.i18n.t("ui.help_nav")),
        ("PgUp / PgDn", app.i18n.t("ui.help_page_scroll")),
        ("1 / 2 / 3", app.i18n.t("ui.help_tabs")),
        ("Enter", app.i18n.t("ui.help_use")),
        ("i", app.i18n.t("ui.help_install")),
        ("d / Delete", app.i18n.t("ui.help_delete")),
        ("s", app.i18n.t("ui.help_shared")),
        ("u", app.i18n.t("ui.help_unuse")),
        ("/", app.i18n.t("ui.help_search")),
        ("Esc", app.i18n.t("ui.help_search_clear")),
        ("r", app.i18n.t("ui.help_refresh")),
        ("m", app.i18n.t("ui.help_storage")),
        ("l", app.i18n.t("ui.help_lang")),
        ("? / h", app.i18n.t("ui.help_toggle")),
        ("q / Ctrl+C", app.i18n.t("ui.help_quit")),
    ];

    let mut lines = Vec::new();
    lines.push(Line::raw(""));

    for (k, desc) in shortcuts {
        lines.push(Line::from(vec![
            Span::styled(format!("{:>14}  ", k), Style::default().fg(Color::Yellow).bold()),
            Span::styled(desc, Style::default().fg(Color::White)),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        app.i18n.t("ui.help_close_hint"),
        Style::default().fg(Color::DarkGray),
    ));

    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

