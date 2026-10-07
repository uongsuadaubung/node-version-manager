use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::NvmApp;
use crate::utils;
use crate::version_service::NodeVersion;

pub fn draw_details_panel(f: &mut Frame, app: &NvmApp, version: Option<NodeVersion>, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", app.i18n.t("ui.details_title")))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(v) = version {
        let is_installed = app.config.installed_versions.contains(&v.version);
        let is_in_use = app.config.current_version.as_deref() == Some(&v.version);
        let is_shared = app
            .config
            .version_configs
            .get(&v.version)
            .copied()
            .unwrap_or(false);

        let dir_name = utils::get_version_dir_name(&v.version);
        let install_path = app.config.versions_dir().join(&dir_name);

        let node_bin = if cfg!(windows) {
            install_path.join("node.exe")
        } else {
            install_path.join("bin").join("node")
        };
        let npm_bin = if cfg!(windows) {
            install_path.join("npm.cmd")
        } else {
            install_path.join("bin").join("npm")
        };

        let mut lines = Vec::new();

        // Version Title
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", app.i18n.t("ui.details_version")), Style::default().fg(Color::DarkGray)),
            Span::styled(v.version.clone(), Style::default().fg(Color::Cyan).bold()),
        ]));

        // LTS info
        let lts_info = if let Some(name) = v.lts_name() {
            app.i18n.t("ui.details_lts_yes").replace("{}", name)
        } else {
            app.i18n.t("ui.details_lts_no").to_string()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", app.i18n.t("ui.details_lts")), Style::default().fg(Color::DarkGray)),
            Span::styled(lts_info, Style::default().fg(Color::Yellow)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(format!("{}: ", app.i18n.t("ui.date_col")), Style::default().fg(Color::DarkGray)),
            Span::styled(v.date.clone(), Style::default().fg(Color::White)),
        ]));

        lines.push(Line::raw(""));

        // Status
        let (status_text, status_style) = if is_in_use {
            (
                format!("● {}", app.i18n.t("ui.in_use")),
                Style::default().fg(Color::LightGreen).bold(),
            )
        } else if is_installed {
            (
                format!("✓ {}", app.i18n.t("ui.downloaded")),
                Style::default().fg(Color::Cyan).bold(),
            )
        } else {
            (
                format!("○ {}", app.i18n.t("ui.not_downloaded")),
                Style::default().fg(Color::DarkGray),
            )
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{}: ", app.i18n.t("ui.status_col")), Style::default().fg(Color::DarkGray)),
            Span::styled(status_text, status_style),
        ]));

        // Directory
        lines.push(Line::from(vec![
            Span::styled(format!("{}:", app.i18n.t("ui.details_path")), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}", install_path.to_string_lossy()),
                if is_installed {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]));

        lines.push(Line::raw(""));

        // Binaries check
        if is_installed {
            lines.push(Line::from(vec![
                Span::styled(format!("{}:", app.i18n.t("ui.details_binaries")), Style::default().fg(Color::DarkGray)),
            ]));
            let node_status = if node_bin.exists() { "✓ node" } else { "✗ node" };
            let npm_status = if npm_bin.exists() { "✓ npm" } else { "✗ npm" };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(node_status, if node_bin.exists() { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }),
                Span::raw("   "),
                Span::styled(npm_status, if npm_bin.exists() { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }),
            ]));

            lines.push(Line::raw(""));

            // Shared NPM mode
            lines.push(Line::from(vec![
                Span::styled(format!("{}:", app.i18n.t("ui.details_shared_mode")), Style::default().fg(Color::DarkGray)),
            ]));
            if is_shared {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(app.i18n.t("ui.details_shared_on"), Style::default().fg(Color::LightGreen)),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("-> {}", app.config.modules_dir().to_string_lossy()), Style::default().fg(Color::DarkGray)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(app.i18n.t("ui.details_shared_off"), Style::default().fg(Color::Yellow)),
                ]));
            }
        }

        lines.push(Line::raw(""));
        lines.push(Line::styled("─".repeat(inner_area.width as usize), Style::default().fg(Color::DarkGray)));

        // Action Hints
        lines.push(Line::styled(app.i18n.t("ui.details_actions_title"), Style::default().fg(Color::Cyan).bold()));
        if is_in_use {
            lines.push(Line::from(vec![
                Span::styled("[u] ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(app.i18n.t("ui.unuse_btn"), Style::default().fg(Color::White)),
            ]));
        } else if is_installed {
            lines.push(Line::from(vec![
                Span::styled("[Enter] ", Style::default().fg(Color::LightGreen).bold()),
                Span::styled(app.i18n.t("ui.use_btn"), Style::default().fg(Color::White)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("[Enter / i] ", Style::default().fg(Color::Cyan).bold()),
                Span::styled(app.i18n.t("ui.details_action_install_use"), Style::default().fg(Color::White)),
            ]));
        }

        if is_installed {
            lines.push(Line::from(vec![
                Span::styled("[s] ", Style::default().fg(Color::Magenta).bold()),
                Span::styled(app.i18n.t("ui.details_action_toggle_shared"), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("[d] ", Style::default().fg(Color::Red).bold()),
                Span::styled(app.i18n.t("ui.delete_btn"), Style::default().fg(Color::White)),
            ]));
        }

        let para = Paragraph::new(lines).wrap(Wrap { trim: true });
        f.render_widget(para, inner_area);
    } else {
        let no_data = Paragraph::new(app.i18n.t("ui.no_matching_versions"))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, inner_area);
    }
}
