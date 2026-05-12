use std::path::PathBuf;
use std::thread;

use super::file_ops::move_task;

use super::{AppMessage, NvmApp, StorageMsg};

impl NvmApp {
    pub(super) fn move_storage(&mut self, path: PathBuf) {
        self.is_loading = true;
        self.status_msg = self.i18n.t("status.preparing_move");
        self.error = None;
        let tx = self.tx.clone();
        let old_base = self.config.base_dir.clone();
        let installed_versions = self.config.installed_versions.clone();
        let lang = self.config.language.clone();

        thread::spawn(move || {
            if let Err(e) = move_task(path, old_base, installed_versions, lang, tx.clone()) {
                tx.send(AppMessage::Storage(StorageMsg::Error(e.to_string()))).ok();
            }
        });
    }
}
