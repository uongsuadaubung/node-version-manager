pub fn platform_suffix() -> &'static str {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    };
    match std::env::consts::OS {
        "windows" => match arch {
            "arm64" => "win-arm64",
            _ => "win-x64",
        },
        "macos" => match arch {
            "arm64" => "darwin-arm64",
            _ => "darwin-x64",
        },
        _ => match arch {
            "arm64" => "linux-arm64",
            _ => "linux-x64",
        },
    }
}

/// Chuẩn hóa đường dẫn để so sánh (Chỉ dùng cho Windows Registry logic)
#[cfg(windows)]
pub fn normalize_path(p: &str) -> String {
    p.replace("/", "\\").trim_end_matches('\\').to_lowercase()
}

pub fn get_version_dir_name(version: &str) -> String {
    format!("node-{}-{}", version, platform_suffix())
}
