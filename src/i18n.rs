use std::collections::HashMap;

pub struct I18n {
    strings: HashMap<String, String>,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        let content = if lang == "en" {
            include_str!("../locales/en.toml")
        } else {
            include_str!("../locales/vi.toml")
        };

        let mut strings = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_string();
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let mut val = v.trim();
                if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                    val = &val[1..val.len() - 1];
                }
                let unescaped = val.replace("\\\"", "\"").replace("\\\\", "\\");
                let full_key = if current_section.is_empty() {
                    key.to_string()
                } else {
                    format!("{}.{}", current_section, key)
                };
                strings.insert(full_key, unescaped);
            }
        }

        Self { strings }
    }

    pub fn t(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}
