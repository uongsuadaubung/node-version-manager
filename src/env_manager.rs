use std::ptr;
use winreg::enums::*;
use winreg::RegKey;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
use windows::Win32::Foundation::{WPARAM, LPARAM};
use std::path::PathBuf;
use std::fs;
use directories::UserDirs;

/// Cập nhật PATH của User trong Registry
pub fn update_user_path(node_dir: &PathBuf, modules_dir: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    
    let current_path_str: String = env.get_value("Path").unwrap_or_else(|_| String::new());
    let mut paths: Vec<String> = current_path_str.split(';').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
    
    // Xóa các đường dẫn cũ liên quan đến .nvm-rust để tránh trùng lặp hoặc rác
    paths.retain(|p| !p.contains(".nvm-rust"));
    
    // Thêm đường dẫn Node mới
    paths.insert(0, node_dir.to_string_lossy().to_string());
    
    // Thêm đường dẫn Modules nếu được bật
    if let Some(m_dir) = modules_dir {
        paths.insert(1, m_dir.to_string_lossy().to_string());
    }
    
    let new_path = paths.join(";");
    env.set_value("Path", &new_path)?;
    
    broadcast_setting_change();
    Ok(())
}

/// Thông báo cho Windows rằng biến môi trường đã thay đổi
fn broadcast_setting_change() {
    let env_str: Vec<u16> = "Environment\0".encode_utf16().collect();
    unsafe {
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(env_str.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            Some(ptr::null_mut()),
        );
    }
}

/// Cập nhật file .npmrc để cấu hình prefix
pub fn update_npmrc(modules_path: &PathBuf, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let user_dirs = UserDirs::new().ok_or("Could not find home directory")?;
    let npmrc_path = user_dirs.home_dir().join(".npmrc");
    
    let content = if npmrc_path.exists() {
        fs::read_to_string(&npmrc_path)?
    } else {
        String::new()
    };
    
    let mut lines: Vec<String> = content.lines()
        .filter(|l| !l.trim().starts_with("prefix="))
        .map(|s| s.to_string())
        .collect();
        
    if enabled {
        // npm yêu cầu đường dẫn dùng dấu gạch chéo xuôi (/) hoặc escape gạch chéo ngược
        let prefix_line = format!("prefix={}", modules_path.to_string_lossy().replace("\\", "/"));
        lines.push(prefix_line);
    }
    
    fs::write(npmrc_path, lines.join("\n"))?;
    Ok(())
}
