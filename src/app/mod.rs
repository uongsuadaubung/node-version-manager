pub mod ui;
pub mod version;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::config::AppConfig;
use crate::i18n::I18n;
use crate::version_service::NodeVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    All = 0,
    Lts = 1,
    Installed = 2,
}

pub struct NvmApp {
    pub config: AppConfig,
    pub versions: Vec<NodeVersion>,
    pub selected_index: usize,
    pub active_tab: Tab,
    pub search_query: String,
    pub is_searching: bool,
    pub is_loading: bool,
    pub download_progress: Option<(u64, u64)>,
    pub download_version: Option<String>,
    pub auto_switch_on_install: Option<String>,
    pub status_msg: String,
    pub error: Option<String>,
    pub confirm_delete: Option<String>,
    pub is_changing_storage: bool,
    pub storage_input: String,
    pub show_help: bool,
    pub should_quit: bool,
    pub tx: Sender<AppMessage>,
    pub rx: Receiver<AppMessage>,
    pub i18n: I18n,
}

#[allow(dead_code)]
pub enum GeneralMsg {
    StatusUpdate(String),
}

pub enum FetchMsg {
    Success(Vec<NodeVersion>),
    Error(String),
}

pub enum DownloadMsg {
    Progress(u64, u64),
    Finished(String),
    Error(String),
}

#[allow(dead_code)]
pub enum AppMessage {
    General(GeneralMsg),
    Fetch(FetchMsg),
    Download(DownloadMsg),
}

#[allow(dead_code)]
pub enum Action {
    Refresh,
    UpdateConfig,
    Switch(String),
    Install(String),
    InstallAndSwitch(String),
    Uninstall(String),
    ChangeLanguage(String),
    Unuse,
}

impl Default for NvmApp {
    fn default() -> Self {
        Self::new()
    }
}

impl NvmApp {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();

        std::thread::spawn(move || {
            let msg = match crate::version_service::fetch_node_versions() {
                Ok(v) => AppMessage::Fetch(FetchMsg::Success(v)),
                Err(e) => AppMessage::Fetch(FetchMsg::Error(e.to_string())),
            };
            tx_clone.send(msg).ok();
        });

        let config = AppConfig::load();
        let lang = config.language.clone();
        let i18n = I18n::new(&lang);

        let mut app = Self {
            config,
            versions: Vec::new(),
            selected_index: 0,
            active_tab: Tab::All,
            search_query: String::new(),
            is_searching: false,
            is_loading: true,
            download_progress: None,
            download_version: None,
            auto_switch_on_install: None,
            error: None,
            status_msg: i18n.t("status.ready"),
            confirm_delete: None,
            is_changing_storage: false,
            storage_input: String::new(),
            show_help: false,
            should_quit: false,
            tx,
            rx,
            i18n,
        };

        app.rescan_installed_versions();
        app
    }

    pub fn rescan_installed_versions(&mut self) {
        let v_dir = self.config.versions_dir();
        self.config.installed_versions.clear();

        let Ok(entries) = std::fs::read_dir(&v_dir) else {
            return;
        };

        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            if !is_dir {
                continue;
            }

            let name = entry.file_name();
            let Some(dir_str) = name.to_str() else {
                continue;
            };

            // Format: node-vX.Y.Z-... -> lấy "vX.Y.Z"
            let Some(ver) = dir_str.strip_prefix("node-").and_then(|s| s.split('-').next()) else {
                continue;
            };

            let ver_string = ver.to_string();
            if !self.config.installed_versions.contains(&ver_string) {
                self.config.installed_versions.push(ver_string);
            }
        }

        self.config.save().ok();
        self.adjust_selection();
    }

    pub fn apply_new_storage_path(&mut self) {
        self.is_changing_storage = false;
        let trimmed = self.storage_input.trim().trim_matches('"').trim_matches('\'');
        if trimmed.is_empty() {
            return;
        }

        let new_path = std::path::PathBuf::from(trimmed);
        let old_base = self.config.base_dir.clone();
        if new_path == old_base {
            return;
        }

        if let Err(e) = std::fs::create_dir_all(&new_path) {
            self.error = Some(
                self.i18n
                    .t("status.storage_create_dir_error")
                    .replace("{}", &e.to_string()),
            );
            return;
        }

        self.config.base_dir = new_path;
        self.rescan_installed_versions();
        self.update_config_and_env(Some(&old_base));
        self.status_msg = self.i18n.t("status.storage_updated");
    }

    pub fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::Fetch(fetch_msg) => match fetch_msg {
                    FetchMsg::Success(v) => {
                        self.versions = v;
                        self.is_loading = false;
                        self.adjust_selection();
                    }
                    FetchMsg::Error(e) => {
                        self.error = Some(e);
                        self.is_loading = false;
                    }
                },
                AppMessage::Download(download_msg) => match download_msg {
                    DownloadMsg::Finished(v) => {
                        if !self.config.installed_versions.contains(&v) {
                            self.config.installed_versions.push(v.clone());
                        }
                        self.config.save().ok();
                        self.is_loading = false;
                        self.download_progress = None;
                        self.download_version = None;
                        self.status_msg = self.i18n.t("status.install_success");

                        if self.auto_switch_on_install.as_deref() == Some(&v) {
                            self.auto_switch_on_install = None;
                            self.switch_version(v);
                        }
                        self.adjust_selection();
                    }
                    DownloadMsg::Error(e) => {
                        self.error = Some(e);
                        self.is_loading = false;
                        self.download_progress = None;
                        self.download_version = None;
                        self.auto_switch_on_install = None;
                    }
                    DownloadMsg::Progress(downloaded, total) => {
                        self.download_progress = Some((downloaded, total));
                    }
                },
                AppMessage::General(GeneralMsg::StatusUpdate(s)) => {
                    self.status_msg = s;
                }
            }
        }
    }

    pub fn filtered_versions(&self) -> Vec<&NodeVersion> {
        let q = self.search_query.trim().to_lowercase();
        self.versions
            .iter()
            .filter(|v| {
                match self.active_tab {
                    Tab::All => true,
                    Tab::Lts => v.is_lts(),
                    Tab::Installed => self.config.installed_versions.contains(&v.version),
                }
            })
            .filter(|v| {
                if q.is_empty() {
                    return true;
                }
                let ver_clean = v.version.to_lowercase();
                let ver_num = ver_clean.trim_start_matches('v');
                if ver_clean.contains(&q) || ver_num.contains(&q) {
                    return true;
                }
                if let Some(lts_name) = v.lts_name()
                    && lts_name.to_lowercase().contains(&q)
                {
                    return true;
                }
                false
            })
            .collect()
    }

    pub fn adjust_selection(&mut self) {
        let count = self.filtered_versions().len();
        if count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= count {
            self.selected_index = count.saturating_sub(1);
        }
    }

    pub fn select_next(&mut self) {
        let count = self.filtered_versions().len();
        if count > 0 && self.selected_index + 1 < count {
            self.selected_index += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn select_page_down(&mut self) {
        let count = self.filtered_versions().len();
        if count > 0 {
            self.selected_index = (self.selected_index + 10).min(count - 1);
        }
    }

    pub fn select_page_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(10);
    }

    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn select_last(&mut self) {
        let count = self.filtered_versions().len();
        if count > 0 {
            self.selected_index = count - 1;
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.selected_index = 0;
    }

    pub fn toggle_language(&mut self) {
        let next_lang = if self.config.language == "vi" { "en" } else { "vi" };
        self.config.language = next_lang.to_string();
        self.i18n = I18n::new(next_lang);
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.lang_changed");
    }

    pub fn selected_version(&self) -> Option<NodeVersion> {
        let filtered = self.filtered_versions();
        filtered.get(self.selected_index).copied().cloned()
    }

    #[allow(dead_code)]
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Refresh => self.refresh_versions(),
            Action::UpdateConfig => self.update_config_and_env(None),
            Action::Switch(v) => self.switch_version(v),
            Action::Install(v) => self.install_version(v),
            Action::InstallAndSwitch(v) => self.install_and_switch_version(v),
            Action::Uninstall(v) => self.uninstall_version(v),
            Action::ChangeLanguage(lang) => {
                self.config.language = lang.clone();
                self.i18n = I18n::new(&lang);
                self.update_config_and_env(None);
            }
            Action::Unuse => self.unuse_version(),
        }
    }
}
