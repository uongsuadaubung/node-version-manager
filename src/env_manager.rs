use std::path::PathBuf;
use std::fs;
use directories::UserDirs;
#[cfg(windows)]
use crate::utils;

#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
#[cfg(windows)]
use windows::Win32::Foundation::{WPARAM, LPARAM};

/// Cập nhật PATH của User
pub fn update_user_path(
    node_dir: &PathBuf, 
    modules_dir: Option<&PathBuf>, 
    base_dir: &PathBuf,
    old_base_dir: Option<&PathBuf>
) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
        
        let current_path_str: String = env.get_value("Path").unwrap_or_else(|_| String::new());
        let mut paths: Vec<String> = current_path_str.split(';').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
        
        let node_path = node_dir.to_string_lossy().to_string();
        let modules_path = modules_dir.map(|d| d.to_string_lossy().to_string());
        
        let base_norm = utils::normalize_path(&base_dir.to_string_lossy());
        let old_base_norm = old_base_dir.map(|d| utils::normalize_path(&d.to_string_lossy()));
        let node_norm = utils::normalize_path(&node_path);
        let mod_norm = modules_path.as_ref().map(|p| utils::normalize_path(p));

        paths.retain(|p| {
            let p_norm = utils::normalize_path(p);
            if p_norm == node_norm { return false; }
            if let Some(ref m) = mod_norm {
                if p_norm == *m { return false; }
            }
            if p_norm.contains(&base_norm) { return false; }
            if let Some(ref old) = old_base_norm {
                if p_norm.contains(old) { return false; }
            }
            !p.contains(".nvm-rust")
        });
        
        paths.insert(0, node_path);
        if let Some(m_path) = modules_path {
            paths.insert(1, m_path);
        }
        
        let new_path = paths.join(";");
        env.set_value("Path", &new_path)?;
        broadcast_setting_change();
        Ok(())
    }

    #[cfg(unix)]
    {
        let _ = base_dir;
        let _ = old_base_dir;
        let user_dirs = UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let home = user_dirs.home_dir();
        
        let mut config_path = home.join(".profile");
        if home.join(".zshrc").exists() {
            config_path = home.join(".zshrc");
        } else if home.join(".bashrc").exists() {
            config_path = home.join(".bashrc");
        }

        let content = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };

        let mut lines: Vec<String> = Vec::new();
        let mut in_block = false;
        for line in content.lines() {
            if line.contains("# >>> nvm-rust >>>") {
                in_block = true;
                continue;
            }
            if line.contains("# <<< nvm-rust <<<") {
                in_block = false;
                continue;
            }
            if !in_block {
                lines.push(line.to_string());
            }
        }

        lines.push("# >>> nvm-rust >>>".to_string());
        let node_bin = node_dir.join("bin");
        lines.push(format!("export PATH=\"{}:$PATH\"", node_bin.display()));
        if let Some(m_dir) = modules_dir {
            lines.push(format!("export PATH=\"{}:$PATH\"", m_dir.display()));
        }
        lines.push("# <<< nvm-rust <<<".to_string());

        fs::write(config_path, lines.join("\n") + "\n")?;
        Ok(())
    }
}

#[cfg(windows)]
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
        let prefix_val = if cfg!(windows) {
            modules_path.to_string_lossy().replace("\\", "/")
        } else {
            modules_path.to_string_lossy().to_string()
        };
        lines.push(format!("prefix={}", prefix_val));
    }
    
    fs::write(npmrc_path, lines.join("\n") + "\n")?;
    Ok(())
}
