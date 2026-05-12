use crate::utils;
use std::path::Path;
use std::ptr;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use winreg::RegKey;
use winreg::enums::*;

pub fn update_user_path(
    node_dir: Option<&Path>,
    modules_dir: Option<&Path>,
    base_dir: &Path,
    old_base_dir: Option<&Path>,
) -> anyhow::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let current_path_str: String = env.get_value("Path").unwrap_or_else(|_| String::new());
    let mut paths: Vec<String> = current_path_str
        .split(';')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let node_path = node_dir.map(|d| d.to_string_lossy().to_string());
    let modules_path = modules_dir.map(|d| d.to_string_lossy().to_string());

    let base_norm = utils::normalize_path(&base_dir.to_string_lossy());
    let old_base_norm = old_base_dir.map(|d| utils::normalize_path(&d.to_string_lossy()));
    let node_norm = node_path.as_ref().map(|p| utils::normalize_path(p));
    let mod_norm = modules_path.as_ref().map(|p| utils::normalize_path(p));

    paths.retain(|p| {
        let p_norm = utils::normalize_path(p);
        if let Some(ref n) = node_norm && p_norm == *n {
            return false;
        }
        if let Some(ref m) = mod_norm && p_norm == *m {
            return false;
        }
        if p_norm.contains(&base_norm) {
            return false;
        }
        if let Some(ref old) = old_base_norm && p_norm.contains(old) {
            return false;
        }
        true
    });

    if let Some(n_path) = node_path {
        paths.insert(0, n_path);
    }
    if let Some(m_path) = modules_path {
        if node_dir.is_some() {
            paths.insert(1, m_path);
        } else {
            paths.insert(0, m_path);
        }
    }

    let new_path = paths.join(";");
    env.set_value("Path", &new_path)?;
    broadcast_setting_change();
    Ok(())
}

fn broadcast_setting_change() {
    let env_str: Vec<u16> = "Environment\0".encode_utf16().collect();
    unsafe {
        SendMessageTimeoutW(
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
