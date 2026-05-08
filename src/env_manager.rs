use std::ptr;
use winreg::enums::*;
use winreg::RegKey;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
use windows::Win32::Foundation::{WPARAM, LPARAM};
use std::path::PathBuf;
use std::fs;
use directories::UserDirs;

/// Cập nhật PATH của User trong Registry
pub fn update_user_path(
    node_dir: &PathBuf, 
    modules_dir: Option<&PathBuf>, 
    base_dir: &PathBuf,
    old_base_dir: Option<&PathBuf>
) -> anyhow::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    
    let current_path_str: String = env.get_value("Path").unwrap_or_else(|_| String::new());
    let mut paths: Vec<String> = current_path_str.split(';').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
    
    let node_path = node_dir.to_string_lossy().to_string();
    let modules_path = modules_dir.map(|d| d.to_string_lossy().to_string());
    let base_path_str = base_dir.to_string_lossy().to_lowercase().replace("/", "\\");
    let old_base_path_str = old_base_dir.map(|d| d.to_string_lossy().to_lowercase().replace("/", "\\"));

    // Chuẩn hóa để so sánh
    let normalize = |p: &str| p.replace("/", "\\").trim_end_matches('\\').to_lowercase();
    let node_norm = normalize(&node_path);
    let mod_norm = modules_path.as_ref().map(|p| normalize(p));

    // Xóa các đường dẫn cũ:
    paths.retain(|p| {
        let p_norm = normalize(p);
        
        // 1. Trùng chính xác với đường dẫn sắp thêm
        if p_norm == node_norm { return false; }
        if let Some(ref m) = mod_norm {
            if p_norm == *m { return false; }
        }
        
        // 2. Nằm trong thư mục base_dir hiện tại
        if p_norm.contains(&base_path_str) { return false; }

        // 3. Nằm trong thư mục base_dir CŨ (nếu có truyền vào)
        if let Some(ref old_base) = old_base_path_str {
            if p_norm.contains(old_base) { return false; }
        }

        // 4. Tương thích ngược với folder mặc định .nvm-rust
        !p.contains(".nvm-rust")
    });
    
    // Thêm đường dẫn mới vào đầu
    paths.insert(0, node_path);
    if let Some(m_path) = modules_path {
        paths.insert(1, m_path);
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
pub fn update_npmrc(modules_path: &PathBuf, enabled: bool) -> anyhow::Result<()> {
    let user_dirs = UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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
