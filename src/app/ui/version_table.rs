use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};

use crate::app::NvmApp;
use super::details_panel::draw_details_panel;

pub fn draw_main_content(f: &mut Frame, app: &mut NvmApp, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(63), // Table list
            Constraint::Percentage(37), // Details panel
        ])
        .split(area);

    let filtered = app.filtered_versions();
    let total_filtered = filtered.len();

    // Table rows
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let is_selected = i == app.selected_index;
            let is_in_use = app.config.current_version.as_deref() == Some(&v.version);
            let is_installed = app.config.installed_versions.contains(&v.version);
            let is_shared = app
                .config
                .version_configs
                .get(&v.version)
                .copied()
                .unwrap_or(false);

            let cursor_col = if is_selected { "▶" } else { " " };

            let ver_style = if is_in_use {
                Style::default().fg(Color::LightGreen).bold()
            } else if is_installed {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };

            let type_cell = if let Some(lts_name) = v.lts_name() {
                Cell::from(format!("LTS ({})", lts_name))
                    .style(Style::default().fg(Color::Yellow))
            } else {
                Cell::from(app.i18n.t("ui.type_current")).style(Style::default().fg(Color::Magenta))
            };

            let status_cell = if is_in_use {
                Cell::from(format!("● {}", app.i18n.t("ui.in_use")))
                    .style(Style::default().fg(Color::LightGreen).bold())
            } else if is_installed {
                Cell::from(format!("✓ {}", app.i18n.t("ui.downloaded")))
                    .style(Style::default().fg(Color::Cyan))
            } else {
                Cell::from(format!("○ {}", app.i18n.t("ui.not_downloaded")))
                    .style(Style::default().fg(Color::DarkGray))
            };

            let shared_cell = if is_installed {
                if is_shared {
                    Cell::from(app.i18n.t("ui.shared_badge")).style(Style::default().fg(Color::LightGreen))
                } else {
                    Cell::from(app.i18n.t("ui.isolated_badge")).style(Style::default().fg(Color::Gray))
                }
            } else {
                Cell::from("-").style(Style::default().fg(Color::DarkGray))
            };

            let row_cells = vec![
                Cell::from(cursor_col).style(Style::default().fg(Color::Yellow).bold()),
                Cell::from(v.version.clone()).style(ver_style),
                type_cell,
                status_cell,
                shared_cell,
                Cell::from(v.date.clone()).style(Style::default().fg(Color::DarkGray)),
            ];

            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(28, 38, 55))
            } else {
                Style::default()
            };

            Row::new(row_cells).style(row_style)
        })
        .collect();

    let header_row = Row::new(vec![
        Cell::from(" "),
        Cell::from(app.i18n.t("ui.version_col")).style(Style::default().fg(Color::Cyan).bold()),
        Cell::from(app.i18n.t("ui.type_col")).style(Style::default().fg(Color::Cyan).bold()),
        Cell::from(app.i18n.t("ui.status_col")).style(Style::default().fg(Color::Cyan).bold()),
        Cell::from(app.i18n.t("ui.shared_col")).style(Style::default().fg(Color::Cyan).bold()),
        Cell::from(app.i18n.t("ui.date_col")).style(Style::default().fg(Color::Cyan).bold()),
    ])
    .style(Style::default().bg(Color::Rgb(15, 20, 30)));

    let cur_idx = if total_filtered == 0 { 0 } else { app.selected_index + 1 };
    let table_title = app
        .i18n
        .t("ui.table_title")
        .replace("{cur}", &cur_idx.to_string())
        .replace("{total}", &total_filtered.to_string());

    let widths = [
        Constraint::Length(2),  // Cursor
        Constraint::Length(12), // Version
        Constraint::Length(14), // Type/LTS
        Constraint::Length(14), // Status
        Constraint::Length(12), // Shared
        Constraint::Length(11), // Date
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .title(table_title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);

    let mut state = TableState::default();
    if total_filtered > 0 {
        state.select(Some(app.selected_index));
    }
    f.render_stateful_widget(table, main_chunks[0], &mut state);

    // Details panel
    let selected_ver = filtered.get(app.selected_index).copied().cloned();
    draw_details_panel(f, app, selected_ver, main_chunks[1]);
}
