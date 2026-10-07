#[path = "compat/directories.rs"]
mod directories;

#[path = "compat/anyhow.rs"]
#[macro_use]
mod anyhow;

#[cfg(windows)]
#[path = "compat/winreg.rs"]
mod winreg;

#[cfg(windows)]
#[path = "compat/windows.rs"]
mod windows;

mod app;
mod config;
mod downloader;
mod env_manager;
mod i18n;
mod utils;
mod version_service;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{NvmApp, Tab, ui};

fn main() -> anyhow::Result<()> {
    // Thiết lập panic hook để luôn trả lại terminal bình thường khi gặp crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = NvmApp::new();

    let res = run_app(&mut terminal, &mut app);

    // Dọn dẹp trả lại terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Err(e) = res {
        eprintln!("Error: {:?}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut NvmApp,
) -> anyhow::Result<()> {
    loop {
        // Nhận dữ liệu từ background thread
        app.handle_messages();

        // Vẽ UI
        terminal.draw(|f| ui::draw_ui(f, app))?;

        if app.should_quit {
            break;
        }

        // Đợi phím hoặc cập nhật UI theo chu kỳ 50ms (mượt mà, không tốn CPU)
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Paste(text) => handle_paste(app, &text),
                Event::Key(key) if key.kind == KeyEventKind::Press => handle_key_event(app, key),
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_paste(app: &mut NvmApp, text: &str) {
    let clean = text.trim().trim_matches('"').trim_matches('\'');
    if app.is_changing_storage {
        app.storage_input.push_str(clean);
    } else if app.is_searching {
        app.search_query.push_str(clean);
        app.adjust_selection();
    }
}

fn handle_key_event(app: &mut NvmApp, key: event::KeyEvent) {
    // 1. Modal xác nhận xóa
    if app.confirm_delete.is_some() {
        handle_confirm_delete_key(app, key.code);
        return;
    }

    // 2. Popup trợ giúp
    if app.show_help {
        handle_help_modal_key(app, key.code);
        return;
    }

    // 3. Modal đổi nơi lưu trữ
    if app.is_changing_storage {
        handle_storage_modal_key(app, key);
        return;
    }

    // 4. Chế độ tìm kiếm
    if app.is_searching {
        handle_search_key(app, key.code);
        return;
    }

    // 5. Điều khiển bình thường
    handle_normal_key(app, key);
}

fn handle_confirm_delete_key(app: &mut NvmApp, code: KeyCode) {
    let Some(ver) = app.confirm_delete.clone() else { return; };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.uninstall_version(ver);
            app.confirm_delete = None;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_delete = None;
        }
        _ => {}
    }
}

fn handle_help_modal_key(app: &mut NvmApp, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter => {
            app.show_help = false;
        }
        _ => {}
    }
}

fn handle_storage_modal_key(app: &mut NvmApp, key: event::KeyEvent) {
    match key.code {
        KeyCode::Esc => app.is_changing_storage = false,
        KeyCode::Enter => app.apply_new_storage_path(),
        KeyCode::Backspace => { app.storage_input.pop(); }
        KeyCode::Char('u') | KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.storage_input.clear();
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.storage_input.push(c);
        }
        _ => {}
    }
}

fn handle_search_key(app: &mut NvmApp, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.search_query.clear();
            app.is_searching = false;
            app.adjust_selection();
        }
        KeyCode::Enter | KeyCode::Down | KeyCode::Up => {
            app.is_searching = false;
            app.adjust_selection();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.adjust_selection();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.adjust_selection();
        }
        _ => {}
    }
}

fn handle_normal_key(app: &mut NvmApp, key: event::KeyEvent) {
    match key.code {
        // Thoát
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,

        // Trợ giúp
        KeyCode::Char('?') | KeyCode::Char('h') => app.show_help = true,

        // Điều hướng danh sách
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::PageUp => app.select_page_up(),
        KeyCode::PageDown => app.select_page_down(),
        KeyCode::Home => app.select_first(),
        KeyCode::End => app.select_last(),

        // Chuyển Tabs
        KeyCode::Char('1') => app.set_tab(Tab::All),
        KeyCode::Char('2') => app.set_tab(Tab::Lts),
        KeyCode::Char('3') => app.set_tab(Tab::Installed),
        KeyCode::Tab => {
            let next_tab = match app.active_tab {
                Tab::All => Tab::Lts,
                Tab::Lts => Tab::Installed,
                Tab::Installed => Tab::All,
            };
            app.set_tab(next_tab);
        }

        // Tìm kiếm & Cài đặt hệ thống
        KeyCode::Char('/') => app.is_searching = true,
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.adjust_selection();
            }
        }
        KeyCode::Char('l') | KeyCode::Char('L') => app.toggle_language(),
        KeyCode::Char('r') | KeyCode::Char('R') => app.refresh_versions(),
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.storage_input.clear();
            app.is_changing_storage = true;
        }
        KeyCode::Char('u') | KeyCode::Char('U') => app.unuse_version(),

        // Thao tác ngữ cảnh trên phiên bản được chọn
        KeyCode::Char('s') | KeyCode::Char('S') => app.toggle_shared_mode(),
        KeyCode::Char('i') | KeyCode::Char('I') => app.install_selected(),
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => app.delete_selected(),
        KeyCode::Enter => app.use_selected(),

        _ => {}
    }
}
