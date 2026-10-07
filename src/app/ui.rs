pub mod delete_modal;
pub mod details_panel;
pub mod footer;
pub mod header;
pub mod help_modal;
pub mod nav_bar;
pub mod status_bar;
pub mod storage_modal;
pub mod utils;
pub mod version_table;

pub use delete_modal::*;
pub use footer::*;
pub use header::*;
pub use help_modal::*;
pub use nav_bar::*;
pub use status_bar::*;
pub use storage_modal::*;
pub use version_table::*;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::NvmApp;

pub fn draw_ui(f: &mut Frame, app: &mut NvmApp) {
    let size = f.area();

    // Phân chia layout chính
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header bar
            Constraint::Length(3), // Tabs & Search bar
            Constraint::Min(8),    // Main list & Details
            Constraint::Length(3), // Status & Progress gauge
            Constraint::Length(2), // Footer keybindings (2 tiers)
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    draw_nav_bar(f, app, chunks[1]);
    draw_main_content(f, app, chunks[2]);
    draw_status_bar(f, app, chunks[3]);
    draw_footer(f, app, chunks[4]);

    // Vẽ modal đè lên nếu có
    if let Some(ref ver) = app.confirm_delete {
        draw_confirm_delete_modal(f, app, ver);
    } else if app.is_changing_storage {
        draw_change_storage_modal(f, app);
    } else if app.show_help {
        draw_help_modal(f, app);
    }
}
