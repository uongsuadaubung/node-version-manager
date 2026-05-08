pub const PLATFORM_SUFFIX: &str = if cfg!(windows) { "win-x64" } else { "linux-x64" };

/// Chuẩn hóa đường dẫn để so sánh (Chỉ dùng cho Windows Registry logic)
#[cfg(windows)]
pub fn normalize_path(p: &str) -> String {
    p.replace("/", "\\").trim_end_matches('\\').to_lowercase()
}

pub fn get_version_dir_name(version: &str) -> String {
    format!("node-{}-{}", version, PLATFORM_SUFFIX)
}
