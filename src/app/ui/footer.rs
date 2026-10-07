use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::NvmApp;

pub fn draw_footer(f: &mut Frame, app: &NvmApp, area: Rect) {
    // --- Tầng 1: Hành động ngữ cảnh theo State của phiên bản đang chọn ---
    let mut state_spans = Vec::new();
    state_spans.push(Span::styled(
        app.i18n.t("ui.actions_label"),
        Style::default().fg(Color::Yellow).bold(),
    ));

    if let Some(v) = app.selected_version() {
        let is_installed = app.config.installed_versions.contains(&v.version);
        let is_in_use = app.config.current_version.as_deref() == Some(&v.version);

        let mut actions: Vec<(&str, String, Color)> = Vec::new();
        if !is_installed {
            // Chưa cài đặt: hiện [Enter] Cài & Dùng, [i] Cài đặt (KHÔNG hiện d, s, u)
            actions.push(("Enter", app.i18n.t("ui.btn_install_and_use"), Color::LightGreen));
            actions.push(("i", app.i18n.t("ui.btn_install_short"), Color::Cyan));
        } else {
            // Đã cài đặt:
            if is_in_use {
                // Đang dùng: hiện [u] Bỏ dùng
                actions.push(("u", app.i18n.t("ui.btn_unuse_short"), Color::Yellow));
            } else {
                // Đã cài nhưng chưa kích hoạt: hiện [Enter] Dùng
                actions.push(("Enter", app.i18n.t("ui.btn_use_short"), Color::LightGreen));
            }
            // Đã cài đặt thì có thể xóa hoặc đổi chế độ modules
            actions.push(("d", app.i18n.t("ui.btn_delete_short"), Color::Red));
            actions.push(("s", app.i18n.t("ui.btn_toggle_shared"), Color::Magenta));
        }

        for (i, (key, desc, color)) in actions.iter().enumerate() {
            if i > 0 {
                state_spans.push(Span::raw("   "));
            }
            state_spans.push(Span::styled(format!("[{}]", key), Style::default().fg(*color).bold()));
            state_spans.push(Span::styled(format!(":{}", desc), Style::default().fg(Color::White)));
        }
    } else {
        state_spans.push(Span::styled(
            app.i18n.t("ui.no_version_selected"),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // --- Tầng 2: Menu điều khiển mặc định (luôn luôn hiển thị) ---
    let default_keys: Vec<(&str, String)> = vec![
        ("↑/↓", app.i18n.t("ui.nav_select")),
        ("1-3", app.i18n.t("ui.nav_tab")),
        ("/", app.i18n.t("ui.nav_search")),
        ("r", app.i18n.t("ui.nav_refresh")),
        ("m", app.i18n.t("ui.btn_storage")),
        ("l", app.i18n.t("ui.nav_lang")),
        ("?", app.i18n.t("ui.nav_help")),
        ("q", app.i18n.t("ui.nav_quit")),
    ];

    let mut default_spans = Vec::new();
    default_spans.push(Span::styled(
        app.i18n.t("ui.controls_label"),
        Style::default().fg(Color::DarkGray),
    ));
    for (i, (key, desc)) in default_keys.iter().enumerate() {
        if i > 0 {
            default_spans.push(Span::raw("  "));
        }
        default_spans.push(Span::styled(
            format!("[{}]", key),
            if *key == "q" {
                Style::default().fg(Color::DarkGray).bold()
            } else {
                Style::default().fg(Color::Cyan).bold()
            },
        ));
        default_spans.push(Span::styled(format!(":{}", desc), Style::default().fg(Color::Gray)));
    }

    let footer_para = Paragraph::new(vec![
        Line::from(state_spans),
        Line::from(default_spans),
    ])
    .alignment(Alignment::Center);

    f.render_widget(footer_para, area);
}
