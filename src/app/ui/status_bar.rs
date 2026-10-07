use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
};

use crate::app::NvmApp;

pub fn draw_status_bar(f: &mut Frame, app: &NvmApp, area: Rect) {
    if let Some((downloaded, total)) = app.download_progress {
        let pct = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0) as u16
        } else {
            0
        };

        let ver_name = app.download_version.as_deref().unwrap_or("Node.js");
        let dl_mb = downloaded as f64 / (1024.0 * 1024.0);
        let total_mb = total as f64 / (1024.0 * 1024.0);

        let label = app
            .i18n
            .t("ui.status_downloading_progress")
            .replace("{ver}", ver_name)
            .replace("{dl}", &format!("{:.1}", dl_mb))
            .replace("{total}", &format!("{:.1}", total_mb))
            .replace("{pct}", &pct.to_string());

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Rgb(20, 25, 40)))
            .percent(pct.min(100))
            .label(label);
        f.render_widget(gauge, area);
    } else {
        let (status_text, border_color, text_color) = if let Some(ref err) = app.error {
            (
                app.i18n.t("ui.status_error_prefix").replace("{}", err),
                Color::Red,
                Color::LightRed,
            )
        } else if app.is_loading {
            (
                format!(" ⏳ {}", app.status_msg),
                Color::Yellow,
                Color::Yellow,
            )
        } else {
            (
                format!(" 💬 {}", app.status_msg),
                Color::DarkGray,
                Color::White,
            )
        };

        let status_para = Paragraph::new(status_text)
            .style(Style::default().fg(text_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color)),
            );
        f.render_widget(status_para, area);
    }
}

