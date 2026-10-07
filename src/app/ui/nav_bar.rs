use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
};

use crate::app::{NvmApp, Tab};

pub fn draw_nav_bar(f: &mut Frame, app: &NvmApp, area: Rect) {
    let nav_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(area);

    let tab_titles = vec![
        format!("1. {} ({})", app.i18n.t("ui.tab_all"), app.versions.len()),
        format!(
            "2. {} ({})",
            app.i18n.t("ui.tab_lts"),
            app.versions.iter().filter(|v| v.is_lts()).count()
        ),
        format!(
            "3. {} ({})",
            app.i18n.t("ui.tab_installed"),
            app.config.installed_versions.len()
        ),
    ];

    let tab_index = match app.active_tab {
        Tab::All => 0,
        Tab::Lts => 1,
        Tab::Installed => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .select(tab_index)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(" | ");
    f.render_widget(tabs, nav_chunks[0]);

    // Search bar
    let (search_text, border_color) = if app.is_searching {
        (
            format!("🔍 {}{}", app.search_query, "█"),
            Color::Yellow,
        )
    } else if app.search_query.is_empty() {
        (
            format!("🔍 {}", app.i18n.t("ui.search_placeholder")),
            Color::DarkGray,
        )
    } else {
        (
            format!("🔍 {} {}", app.search_query, app.i18n.t("ui.search_active_hint")),
            Color::Cyan,
        )
    };

    let search_para = Paragraph::new(search_text)
        .style(if app.is_searching {
            Style::default().fg(Color::Yellow).bold()
        } else if app.search_query.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan)
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );
    f.render_widget(search_para, nav_chunks[1]);
}
