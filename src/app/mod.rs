mod file_ops;
mod storage;
mod ui;
mod version;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::config::AppConfig;
use crate::i18n::I18n;
use crate::version_service::NodeVersion;

pub struct NvmApp {
    pub config: AppConfig,
    pub versions: Vec<NodeVersion>,
    pub is_loading: bool,
    pub lts_only: bool,
    pub search_query: String,
    pub download_progress: Option<(u64, u64)>,
    pub move_progress: Option<(usize, usize)>,
    pub error: Option<String>,
    pub status_msg: String,
    pub tx: Sender<AppMessage>,
    pub rx: Receiver<AppMessage>,
    pub i18n: I18n,
}

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

pub enum StorageMsg {
    Progress(usize, usize),
    Finished(PathBuf),
    Error(String),
}

pub enum AppMessage {
    General(GeneralMsg),
    Fetch(FetchMsg),
    Download(DownloadMsg),
    Storage(StorageMsg),
}

pub enum Action {
    Refresh,
    UpdateConfig,
    Switch(String),
    Install(String),
    Uninstall(String),
    MoveStorage(PathBuf),
    ChangeLanguage(String),
    Unuse,
}

impl NvmApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use eframe::egui;
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        ui::setup_custom_fonts(&cc.egui_ctx);

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

        Self {
            config,
            versions: Vec::new(),
            is_loading: true,
            lts_only: true,
            search_query: String::new(),
            download_progress: None,
            move_progress: None,
            error: None,
            status_msg: i18n.t("status.ready"),
            tx,
            rx,
            i18n,
        }
    }

    pub(crate) fn handle_action(&mut self, action: Action) {
        match action {
            Action::Refresh => self.refresh_versions(),
            Action::UpdateConfig => self.update_config_and_env(None),
            Action::Switch(v) => self.switch_version(v),
            Action::Install(v) => self.install_version(v),
            Action::Uninstall(v) => self.uninstall_version(v),
            Action::MoveStorage(path) => self.move_storage(path),
            Action::ChangeLanguage(lang) => {
                self.config.language = lang.clone();
                self.i18n = I18n::new(&lang);
                self.update_config_and_env(None);
            }
            Action::Unuse => self.unuse_version(),
        }
    }
}
