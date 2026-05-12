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

        if let Some(ref version) = self.config.current_version {
            let version_dir_name = utils::get_version_dir_name(version);
            let version_path = self.config.versions_dir().join(&version_dir_name);

            if version_path.exists() {
                let use_shared = self
                    .config
                    .version_configs
                    .get(version)
                    .cloned()
                    .unwrap_or(false);

                let modules_dir = if use_shared {
                    let m_dir = self.config.modules_dir();
                    if !m_dir.exists() && let Err(e) = std::fs::create_dir_all(&m_dir) {
                        self.error = Some(
                            self.i18n
                                .t("status.create_modules_error")
                                .replace("{}", &e.to_string()),
                        );
                        return;
                    }
                    Some(m_dir)
                } else {
                    None
                };

                if let Err(e) = env_manager::update_user_path(
                    Some(version_path.as_path()),
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
        } else {
            if let Err(e) =
                env_manager::update_user_path(None, None, self.config.base_dir.as_path(), old_base_dir.map(|p| p.as_path()))
            {
                self.error = Some(
                    self.i18n
                        .t("status.update_path_error")
                        .replace("{}", &e.to_string()),
                );
            }
            if let Err(e) = env_manager::update_npmrc(self.config.modules_dir().as_path(), false) {
                self.error = Some(
                    self.i18n
                        .t("status.update_npmrc_error")
                        .replace("{}", &e.to_string()),
                );
            }
        }
    }

    pub(super) fn refresh_versions(&mut self) {
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

    pub(super) fn switch_version(&mut self, v: String) {
        self.config.current_version = Some(v.clone());
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.switched_to").replace("{}", &v);
    }

    pub(super) fn install_version(&mut self, v: String) {
        self.is_loading = true;
        self.move_progress = None;
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

    pub(super) fn uninstall_version(&mut self, v: String) {
        let version_dir_name = utils::get_version_dir_name(&v);
        let version_path = self.config.versions_dir().join(version_dir_name);
        if version_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&version_path) {
                self.error = Some(
                    self.i18n
                        .t("status.delete_dir_error")
                        .replace("{}", &e.to_string()),
                );
            } else {
                self.config.installed_versions.retain(|iv| iv != &v);
                self.config.version_configs.remove(&v);
                if let Err(e) = self.config.save() {
                    self.error = Some(
                        self.i18n
                            .t("status.save_config_after_delete_error")
                            .replace("{}", &e.to_string()),
                    );
                }
                self.status_msg = self.i18n.t("status.deleted_version").replace("{}", &v);
            }
        }
    }

    pub(super) fn unuse_version(&mut self) {
        self.config.current_version = None;
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.unused_version");
    }
}
