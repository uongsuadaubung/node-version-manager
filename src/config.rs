use crate::anyhow;
use crate::directories::UserDirs;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub base_dir: PathBuf,
    pub current_version: Option<String>,
    pub version_configs: HashMap<String, bool>,
    pub installed_versions: Vec<String>,
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

    pub fn from_toml_str(content: &str) -> anyhow::Result<Self> {
        fn strip_quotes(s: &str) -> &str {
            let s = s.trim();
            if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
                if s.len() >= 2 {
                    &s[1..s.len() - 1]
                } else {
                    ""
                }
            } else {
                s
            }
        }

        fn unescape(s: &str) -> String {
            let s = strip_quotes(s);
            s.replace("\\\\", "\\").replace("\\\"", "\"")
        }

        let mut config = Self::default();
        let mut current_section = String::new();
        let mut in_array_key: Option<String> = None;
        let mut array_items: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(ref key) = in_array_key {
                let end = line.find(']');
                let part = if let Some(idx) = end { &line[..idx] } else { line };
                for item in part.split(',') {
                    let item = item.trim();
                    if !item.is_empty() {
                        array_items.push(unescape(item));
                    }
                }
                if end.is_some() {
                    if key == "installed_versions" {
                        config.installed_versions = std::mem::take(&mut array_items);
                    }
                    in_array_key = None;
                }
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_string();
                continue;
            }

            if let Some((k, v)) = line.split_once('=') {
                let k = strip_quotes(k.trim());
                let v = v.trim();

                if current_section == "version_configs" {
                    let val = v.trim().parse::<bool>().unwrap_or(false);
                    config.version_configs.insert(k.to_string(), val);
                } else if k == "base_dir" {
                    config.base_dir = PathBuf::from(unescape(v));
                } else if k == "current_version" {
                    let val = unescape(v);
                    config.current_version = if val.is_empty() || val == "null" {
                        None
                    } else {
                        Some(val)
                    };
                } else if k == "language" {
                    config.language = unescape(v);
                } else if k == "installed_versions"
                    && let Some(start) = v.find('[')
                {
                    if let Some(end) = v.rfind(']') {
                        let items_str = &v[start + 1..end];
                        config.installed_versions = items_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(unescape)
                            .collect();
                    } else {
                        in_array_key = Some(k.to_string());
                        let part = &v[start + 1..];
                        for item in part.split(',') {
                            let item = item.trim();
                            if !item.is_empty() {
                                array_items.push(unescape(item));
                            }
                        }
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn to_toml_string(&self) -> String {
        let mut out = String::new();
        let base_dir_str = self.base_dir.to_string_lossy().replace('\\', "\\\\");
        out.push_str(&format!("base_dir = \"{}\"\n", base_dir_str));
        if let Some(ref ver) = self.current_version {
            out.push_str(&format!("current_version = \"{}\"\n", ver));
        }
        out.push_str(&format!("language = \"{}\"\n", self.language));

        let versions_str: Vec<String> = self
            .installed_versions
            .iter()
            .map(|v| format!("\"{}\"", v))
            .collect();
        out.push_str(&format!("installed_versions = [{}]\n", versions_str.join(", ")));

        out.push_str("\n[version_configs]\n");
        let mut sorted_keys: Vec<_> = self.version_configs.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            let v = self.version_configs[k];
            out.push_str(&format!("\"{}\" = {}\n", k, v));
        }
        out
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::config_file() && path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            return Self::from_toml_str(&content).unwrap_or_else(|_| Self::default());
        }

        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_file()?;
        let content = self.to_toml_string();
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
