use directories::UserDirs;
use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::update_user_path;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::update_user_path;

pub fn update_npmrc(modules_path: &PathBuf, enabled: bool) -> anyhow::Result<()> {
    let user_dirs =
        UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let npmrc_path = user_dirs.home_dir().join(".npmrc");

    let content = if npmrc_path.exists() {
        fs::read_to_string(&npmrc_path)?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = content
        .lines()
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
