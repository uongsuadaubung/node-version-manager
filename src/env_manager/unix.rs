use std::path::PathBuf;
use std::fs;
use directories::UserDirs;

pub fn update_user_path(
    node_dir: Option<&PathBuf>, 
    modules_dir: Option<&PathBuf>, 
    _base_dir: &PathBuf,
    _old_base_dir: Option<&PathBuf>
) -> anyhow::Result<()> {
    let user_dirs = UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = user_dirs.home_dir();
    let app_config_dir = home.join(".nvm-rust");
    
    if !app_config_dir.exists() {
        let _ = fs::create_dir_all(&app_config_dir);
    }

    let current_node_link = app_config_dir.join("current_node");
    let current_modules_link = app_config_dir.join("current_modules");

    // Cập nhật Symlink cho Node
    let _ = fs::remove_file(&current_node_link);
    if let Some(nd) = node_dir {
        let _ = std::os::unix::fs::symlink(nd, &current_node_link);
    }

    // Cập nhật Symlink cho Global Modules
    let _ = fs::remove_file(&current_modules_link);
    if let Some(m_dir) = modules_dir {
        let _ = std::os::unix::fs::symlink(m_dir, &current_modules_link);
    }

    // Cập nhật tĩnh PATH vào các file cấu hình Shell
    for shell_rc in [".bashrc", ".zshrc", ".profile"] {
        let config_path = home.join(shell_rc);
        if !config_path.exists() { continue; }

        let content = fs::read_to_string(&config_path).unwrap_or_default();
        
        // Xóa block cũ nếu có
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

        // Luôn ghi script kiểm tra chống trùng lặp (giống cách rustup làm)
        lines.push("# >>> nvm-rust >>>".to_string());
        lines.push("NVM_RUST_PATH=\"$HOME/.nvm-rust/current_node/bin:$HOME/.nvm-rust/current_modules/bin\"".to_string());
        lines.push("case \":${PATH}:\" in".to_string());
        lines.push("    *\":${NVM_RUST_PATH}:\"*) ;;".to_string());
        lines.push("    *) export PATH=\"${NVM_RUST_PATH}:$PATH\" ;;".to_string());
        lines.push("esac".to_string());
        lines.push("# <<< nvm-rust <<<".to_string());

        let _ = fs::write(config_path, lines.join("\n") + "\n");
    }
    
    Ok(())
}
