use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::UserDirs;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub base_dir: PathBuf,
    pub current_version: Option<String>,
    pub version_configs: HashMap<String, bool>, // Lưu cấu hình: version -> dùng chung global?
    pub installed_versions: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let user_dirs = UserDirs::new().expect("Could not find home directory");
        let base_dir = user_dirs.home_dir().join(".nvm-rust");
        
        AppConfig {
            base_dir,
            current_version: None,
            version_configs: HashMap::new(),
            installed_versions: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn config_file() -> PathBuf {
        let user_dirs = UserDirs::new().expect("Could not find home directory");
        let conf_dir = user_dirs.home_dir().join(".nvm-rust");
        if !conf_dir.exists() {
            let _ = fs::create_dir_all(&conf_dir);
        }
        conf_dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_file();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn versions_dir(&self) -> PathBuf {
        self.base_dir.join("versions")
    }
    
    pub fn modules_dir(&self) -> PathBuf {
        self.base_dir.join("modules")
    }
}
