use std::path::PathBuf;
use std::thread;

use crate::downloader;
use crate::env_manager;
use crate::utils;
use crate::version_service;

use super::{AppMessage, DownloadMsg, FetchMsg, NvmApp};

impl NvmApp {
    pub fn update_config_and_env(&mut self, old_base_dir: Option<&PathBuf>) {
        if let Err(e) = self.config.save() {
            self.error = Some(
                self.i18n
                    .t("status.saving_config_error")
                    .replace("{}", &e.to_string()),
            );
            return;
        }

        let (version_path, modules_dir, use_shared) = self.resolve_active_paths();

        if let Err(e) = env_manager::update_user_path(
            version_path.as_deref(),
            modules_dir.as_deref(),
            self.config.base_dir.as_path(),
            old_base_dir.map(|p| p.as_path()),
        ) {
            self.error = Some(
                self.i18n
                    .t("status.update_path_error")
                    .replace("{}", &e.to_string()),
            );
        }

        if let Err(e) = env_manager::update_npmrc(self.config.modules_dir().as_path(), use_shared) {
            self.error = Some(
                self.i18n
                    .t("status.update_npmrc_error")
                    .replace("{}", &e.to_string()),
            );
        }
    }

    fn resolve_active_paths(&mut self) -> (Option<PathBuf>, Option<PathBuf>, bool) {
        let Some(ref version) = self.config.current_version else {
            return (None, None, false);
        };

        let version_dir_name = utils::get_version_dir_name(version);
        let version_path = self.config.versions_dir().join(&version_dir_name);
        if !version_path.exists() {
            return (None, None, false);
        }

        let use_shared = self
            .config
            .version_configs
            .get(version)
            .copied()
            .unwrap_or(false);

        let modules_dir = if use_shared {
            let m_dir = self.config.modules_dir();
            if !m_dir.exists() && let Err(e) = std::fs::create_dir_all(&m_dir) {
                self.error = Some(
                    self.i18n
                        .t("status.create_modules_error")
                        .replace("{}", &e.to_string()),
                );
                return (Some(version_path), None, use_shared);
            }
            Some(m_dir)
        } else {
            None
        };

        (Some(version_path), modules_dir, use_shared)
    }

    pub fn refresh_versions(&mut self) {
        self.is_loading = true;
        let tx = self.tx.clone();
        thread::spawn(move || {
            let msg = match version_service::fetch_node_versions() {
                Ok(v) => AppMessage::Fetch(FetchMsg::Success(v)),
                Err(e) => AppMessage::Fetch(FetchMsg::Error(e.to_string())),
            };
            tx.send(msg).ok();
        });
    }

    pub fn switch_version(&mut self, v: String) {
        self.config.current_version = Some(v.clone());
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.switched_to").replace("{}", &v);
    }

    pub fn install_version(&mut self, v: String) {
        self.is_loading = true;
        self.download_version = Some(v.clone());
        self.status_msg = self.i18n.t("status.installing").replace("{}", &v);
        let tx = self.tx.clone();
        let base_dir = self.config.versions_dir();
        thread::spawn(move || {
            let msg = match downloader::download_and_extract(&v, &base_dir, &tx) {
                Ok(_) => AppMessage::Download(DownloadMsg::Finished(v)),
                Err(e) => AppMessage::Download(DownloadMsg::Error(e.to_string())),
            };
            tx.send(msg).ok();
        });
    }

    pub fn install_and_switch_version(&mut self, v: String) {
        self.auto_switch_on_install = Some(v.clone());
        self.install_version(v);
    }

    pub fn uninstall_version(&mut self, v: String) {
        let version_dir_name = utils::get_version_dir_name(&v);
        let version_path = self.config.versions_dir().join(version_dir_name);
        if !version_path.exists() {
            return;
        }

        if let Err(e) = std::fs::remove_dir_all(&version_path) {
            self.error = Some(
                self.i18n
                    .t("status.delete_dir_error")
                    .replace("{}", &e.to_string()),
            );
            return;
        }

        self.config.installed_versions.retain(|iv| iv != &v);
        self.config.version_configs.remove(&v);

        if self.config.current_version.as_deref() == Some(&v) {
            self.config.current_version = None;
            self.update_config_and_env(None);
        } else if let Err(e) = self.config.save() {
            self.error = Some(
                self.i18n
                    .t("status.save_config_after_delete_error")
                    .replace("{}", &e.to_string()),
            );
        }

        self.status_msg = self.i18n.t("status.deleted_version").replace("{}", &v);
        self.adjust_selection();
    }

    pub fn unuse_version(&mut self) {
        self.config.current_version = None;
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.unused_version").to_string();
    }

    pub fn toggle_shared_mode(&mut self) {
        let Some(v) = self.selected_version() else { return; };
        if !self.config.installed_versions.contains(&v.version) {
            return;
        }
        let cur = self
            .config
            .version_configs
            .get(&v.version)
            .copied()
            .unwrap_or(false);
        let new_state = !cur;
        self.config.version_configs.insert(v.version.clone(), new_state);
        self.update_config_and_env(None);
        let mode_label = if new_state {
            self.i18n.t("ui.mode_on")
        } else {
            self.i18n.t("ui.mode_off")
        };
        self.status_msg = self
            .i18n
            .t("status.shared_mode_changed")
            .replace("{ver}", &v.version)
            .replace("{mode}", &mode_label);
    }

    pub fn install_selected(&mut self) {
        let Some(v) = self.selected_version() else { return; };
        if !self.is_loading {
            self.install_version(v.version);
        }
    }

    pub fn delete_selected(&mut self) {
        let Some(v) = self.selected_version() else { return; };
        if self.config.installed_versions.contains(&v.version) {
            self.confirm_delete = Some(v.version);
        }
    }

    pub fn use_selected(&mut self) {
        let Some(v) = self.selected_version() else { return; };
        if self.config.installed_versions.contains(&v.version) {
            self.switch_version(v.version);
        } else if !self.is_loading {
            self.install_and_switch_version(v.version);
        }
    }
}
