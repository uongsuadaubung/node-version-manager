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

        let toml_val: toml::Value =
            toml::from_str(content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));
        let mut strings = HashMap::new();

        if let Some(table) = toml_val.as_table() {
            for (section_name, section) in table {
                if let Some(section_table) = section.as_table() {
                    for (k, v) in section_table {
                        if let Some(s) = v.as_str() {
                            strings.insert(format!("{}.{}", section_name, k), s.to_string());
                        }
                    }
                }
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
