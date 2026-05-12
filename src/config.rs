use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub base_dir: PathBuf,
    pub current_version: Option<String>,
    pub version_configs: HashMap<String, bool>,
    pub installed_versions: Vec<String>,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        // Nếu không tìm thấy Home, dùng thư mục hiện tại làm fallback thay vì crash
        let base_dir = UserDirs::new()
            .map(|u| u.home_dir().join(".nvm-rust"))
            .unwrap_or_else(|| PathBuf::from(".nvm-rust"));

        AppConfig {
            base_dir,
            current_version: None,
            version_configs: HashMap::new(),
            installed_versions: Vec::new(),
            language: default_language(),
        }
    }
}

impl AppConfig {
    pub fn config_file() -> anyhow::Result<PathBuf> {
        let user_dirs =
            UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let conf_dir = user_dirs.home_dir().join(".nvm-rust");
        if !conf_dir.exists() {
            fs::create_dir_all(&conf_dir)?;
        }
        Ok(conf_dir.join("config.toml"))
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::config_file() && path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            return toml::from_str(&content).unwrap_or_else(|_| Self::default());
        }

        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_file()?;
        let content = toml::to_string_pretty(self)?;
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
