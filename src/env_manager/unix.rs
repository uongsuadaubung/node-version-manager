use directories::UserDirs;
use std::fs;
use std::path::PathBuf;

pub fn update_user_path(
    node_dir: Option<&PathBuf>,
    modules_dir: Option<&PathBuf>,
    _base_dir: &PathBuf,
    _old_base_dir: Option<&PathBuf>,
) -> anyhow::Result<()> {
    let user_dirs =
        UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = user_dirs.home_dir();
    let farm_dir = home.join(".local").join("bin");

    if !farm_dir.exists() {
        fs::create_dir_all(&farm_dir).ok();
    }

    // Dọn dẹp symlink cũ trong farm_dir (xóa những symlink trỏ về nvm-rust hoặc base_dir cũ/mới)
    if let Ok(entries) = fs::read_dir(&farm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_symlink() {
                if let Ok(target) = fs::read_link(&path) {
                    let is_old_dir = _old_base_dir.map_or(false, |old| target.starts_with(old));
                    let is_new_dir = target.starts_with(_base_dir);

                    if is_old_dir || is_new_dir {
                        fs::remove_file(&path).ok();
                    }
                }
            }
        }
    }

    let create_symlinks = |dir: &PathBuf, overwrite_allowed: bool| {
        let bin_dir = dir.join("bin");
        if bin_dir.exists() {
            if let Ok(entries) = fs::read_dir(&bin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() || path.is_symlink() {
                        if let Some(file_name) = path.file_name() {
                            let link_path = farm_dir.join(file_name);
                            if overwrite_allowed || !link_path.exists() {
                                std::os::unix::fs::symlink(&path, &link_path).ok();
                            }
                        }
                    }
                }
            }
        }
    };

    // Tạo lại symlink cho Node
    if let Some(nd) = node_dir {
        create_symlinks(nd, true);
    }

    // Tạo lại symlink cho Global Modules
    if let Some(m_dir) = modules_dir {
        create_symlinks(m_dir, false);
    }

    Ok(())
}
