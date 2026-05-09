use eframe::egui;
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use std::path::PathBuf;
use std::collections::BTreeMap;

use crate::version_service::{fetch_node_versions, NodeVersion};
use crate::config::AppConfig;
use crate::env_manager;
use crate::downloader;
use crate::version_service;
use crate::utils;

pub struct NvmApp {
    pub config: AppConfig,
    pub versions: Vec<NodeVersion>,
    pub is_loading: bool,
    pub lts_only: bool,
    pub search_query: String,
    pub move_progress: Option<(usize, usize)>,
    pub error: Option<String>,
    pub status_msg: String,
    pub tx: Sender<AppMessage>,
    pub rx: Receiver<AppMessage>,
    pub i18n: crate::i18n::I18n,
}

pub enum AppMessage {
    VersionsFetched(Vec<NodeVersion>),
    FetchError(String),
    InstallFinished(String),
    InstallError(String),
    MoveFinished(PathBuf),
    MoveError(String),
    StatusUpdate(String),
    MoveProgress(usize, usize),
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
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        setup_custom_fonts(&cc.egui_ctx);

        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();

        thread::spawn(move || {
            match fetch_node_versions() {
                Ok(v) => { let _ = tx_clone.send(AppMessage::VersionsFetched(v)); },
                Err(e) => { let _ = tx_clone.send(AppMessage::FetchError(e.to_string())); }
            }
        });

        let config = AppConfig::load();
        let lang = config.language.clone();

        Self {
            config,
            versions: Vec::new(),
            is_loading: true,
            lts_only: true,
            search_query: "".to_string(),
            move_progress: None,
            error: None,
            status_msg: crate::i18n::I18n::new(&lang).t("status.ready"),
            tx,
            rx,
            i18n: crate::i18n::I18n::new(&lang),
        }
    }

    pub fn update_config_and_env(&mut self, old_base_dir: Option<&PathBuf>) {
        if let Err(e) = self.config.save() {
            self.error = Some(self.i18n.t("status.saving_config_error").replace("{}", &e.to_string()));
            return;
        }
        
        if let Some(ref version) = self.config.current_version {
            let version_dir_name = utils::get_version_dir_name(version);
            let version_path = self.config.versions_dir().join(&version_dir_name);
            
            if version_path.exists() {
                let use_shared = self.config.version_configs.get(version).cloned().unwrap_or(false);
                
                let modules_dir = if use_shared {
                    let m_dir = self.config.modules_dir();
                    if !m_dir.exists() {
                        if let Err(e) = std::fs::create_dir_all(&m_dir) {
                            self.error = Some(self.i18n.t("status.create_modules_error").replace("{}", &e.to_string()));
                            return;
                        }
                    }
                    Some(m_dir)
                } else {
                    None
                };
                
                if let Err(e) = env_manager::update_user_path(Some(&version_path), modules_dir.as_ref(), &self.config.base_dir, old_base_dir) {
                    self.error = Some(self.i18n.t("status.update_path_error").replace("{}", &e.to_string()));
                }
                if let Err(e) = env_manager::update_npmrc(&self.config.modules_dir(), use_shared) {
                    self.error = Some(self.i18n.t("status.update_npmrc_error").replace("{}", &e.to_string()));
                }
            }
        } else {
            if let Err(e) = env_manager::update_user_path(None, None, &self.config.base_dir, old_base_dir) {
                self.error = Some(self.i18n.t("status.update_path_error").replace("{}", &e.to_string()));
            }
            if let Err(e) = env_manager::update_npmrc(&self.config.modules_dir(), false) {
                self.error = Some(self.i18n.t("status.update_npmrc_error").replace("{}", &e.to_string()));
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Refresh => self.refresh_versions(),
            Action::UpdateConfig => self.update_config_and_env(None),
            Action::Switch(v) => self.switch_version(v),
            Action::Install(v) => self.install_version(v),
            Action::Uninstall(v) => self.uninstall_version(v),
            Action::MoveStorage(path) => self.move_storage(path),
            Action::ChangeLanguage(lang) => {
                self.config.language = lang.clone();
                self.i18n = crate::i18n::I18n::new(&lang);
                self.update_config_and_env(None);
            }
            Action::Unuse => self.unuse_version(),
        }
    }

    fn refresh_versions(&mut self) {
        self.is_loading = true;
        let tx = self.tx.clone();
        thread::spawn(move || {
            match version_service::fetch_node_versions() {
                Ok(v) => { let _ = tx.send(AppMessage::VersionsFetched(v)); },
                Err(e) => { let _ = tx.send(AppMessage::FetchError(e.to_string())); }
            }
        });
    }

    fn unuse_version(&mut self) {
        self.config.current_version = None;
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.unused_version");
    }

    fn switch_version(&mut self, v: String) {
        self.config.current_version = Some(v.clone());
        self.update_config_and_env(None);
        self.status_msg = self.i18n.t("status.switched_to").replace("{}", &v);
    }

    fn install_version(&mut self, v: String) {
        self.is_loading = true;
        self.status_msg = self.i18n.t("status.installing").replace("{}", &v);
        let tx = self.tx.clone();
        let base_dir = self.config.versions_dir();
        thread::spawn(move || {
            match downloader::download_and_extract(&v, &base_dir) {
                Ok(_) => { let _ = tx.send(AppMessage::InstallFinished(v)); },
                Err(e) => { let _ = tx.send(AppMessage::InstallError(e.to_string())); }
            }
        });
    }

    fn uninstall_version(&mut self, v: String) {
        let version_dir_name = utils::get_version_dir_name(&v);
        let version_path = self.config.versions_dir().join(version_dir_name);
        if version_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&version_path) {
                self.error = Some(self.i18n.t("status.delete_dir_error").replace("{}", &e.to_string()));
            } else {
                self.config.installed_versions.retain(|iv| iv != &v);
                self.config.version_configs.remove(&v);
                if let Err(e) = self.config.save() {
                    self.error = Some(self.i18n.t("status.save_config_after_delete_error").replace("{}", &e.to_string()));
                }
                self.status_msg = self.i18n.t("status.deleted_version").replace("{}", &v);
            }
        }
    }

    fn move_storage(&mut self, path: PathBuf) {
        self.is_loading = true;
        self.status_msg = self.i18n.t("status.preparing_move");
        self.error = None;
        let tx = self.tx.clone();
        let old_base = self.config.base_dir.clone();
        let installed_versions = self.config.installed_versions.clone();
        
        thread::spawn(move || {
            if !path.exists() {
                if let Err(e) = std::fs::create_dir_all(&path) {
                    let _ = tx.send(AppMessage::MoveError(crate::i18n::I18n::new("en").t("status.create_new_dir_error").replace("{}", &e.to_string())));
                    return;
                }
            }

            let mut total_files = 0;
            let versions_from_dir = old_base.join("versions");
            for v in &installed_versions {
                total_files += count_files(&versions_from_dir.join(utils::get_version_dir_name(v)));
            }
            total_files += count_files(&old_base.join("modules"));

            let _ = tx.send(AppMessage::StatusUpdate(crate::i18n::I18n::new("en").t("status.moving_files").replace("{}", &total_files.to_string())));

            let mut copied_count = 0;
            let mut last_update = Instant::now();
            let versions_to_dir = path.join("versions");
            
            if versions_from_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&versions_to_dir) {
                    let _ = tx.send(AppMessage::MoveError(crate::i18n::I18n::new("en").t("status.create_versions_dir_error").replace("{}", &e.to_string())));
                    return;
                }
                for v in installed_versions {
                    let dir_name = utils::get_version_dir_name(&v);
                    let from = versions_from_dir.join(&dir_name);
                    let to = versions_to_dir.join(&dir_name);
                    if from.exists() {
                        if std::fs::rename(&from, &to).is_err() {
                            if let Err(e) = copy_dir_all(&from, &to, &tx, &mut last_update, &mut copied_count, total_files) {
                                let _ = tx.send(AppMessage::MoveError(crate::i18n::I18n::new("en").t("status.copy_error").replacen("{}", &dir_name, 1).replacen("{}", &e.to_string(), 1)));
                                return;
                            }
                            let _ = std::fs::remove_dir_all(&from);
                        } else {
                            copied_count += count_files(&to);
                            let _ = tx.send(AppMessage::MoveProgress(copied_count, total_files));
                        }
                    }
                }
                if is_dir_empty(&versions_from_dir) {
                    let _ = std::fs::remove_dir(&versions_from_dir);
                }
            }

            let modules_from = old_base.join("modules");
            let modules_to = path.join("modules");
            if modules_from.exists() {
                if std::fs::rename(&modules_from, &modules_to).is_err() {
                    if let Err(e) = copy_dir_all(&modules_from, &modules_to, &tx, &mut last_update, &mut copied_count, total_files) {
                        let _ = tx.send(AppMessage::MoveError(crate::i18n::I18n::new("en").t("status.copy_modules_error").replace("{}", &e.to_string())));
                        return;
                    }
                    let _ = std::fs::remove_dir_all(&modules_from);
                } else {
                    copied_count += count_files(&modules_to);
                    let _ = tx.send(AppMessage::MoveProgress(copied_count, total_files));
                }
                if modules_from.exists() && is_dir_empty(&modules_from) {
                    let _ = std::fs::remove_dir(&modules_from);
                }
            }

            let _ = tx.send(AppMessage::MoveFinished(path));
        });
    }
}

impl eframe::App for NvmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::VersionsFetched(v) => {
                    self.versions = v;
                    self.is_loading = false;
                }
                AppMessage::FetchError(e) => {
                    self.error = Some(e);
                    self.is_loading = false;
                }
                AppMessage::InstallFinished(v) => {
                    self.config.installed_versions.push(v);
                    if let Err(e) = self.config.save() {
                        self.error = Some(self.i18n.t("status.saving_config_error").replace("{}", &e.to_string()));
                    }
                    self.status_msg = self.i18n.t("status.install_success");
                    self.is_loading = false;
                }
                AppMessage::InstallError(e) => {
                    self.error = Some(e);
                    self.is_loading = false;
                }
                AppMessage::StatusUpdate(s) => {
                    self.status_msg = s;
                }
                AppMessage::MoveFinished(new_path) => {
                    let old_base = self.config.base_dir.clone();
                    self.config.base_dir = new_path;
                    self.update_config_and_env(Some(&old_base));
                    self.status_msg = self.i18n.t("status.move_success");
                    self.is_loading = false;
                    self.move_progress = None;
                }
                AppMessage::MoveProgress(copied, total) => {
                    self.move_progress = Some((copied, total));
                }
                AppMessage::MoveError(e) => {
                    self.error = Some(e);
                    self.is_loading = false;
                    self.move_progress = None;
                }
            }
        }

        let mut pending_action = None;

        ui.vertical(|ui| {
            ui.heading("Node Version Manager");
            
            ui.horizontal(|ui| {
                ui.label(self.i18n.t("ui.current_version").replace("{}", self.config.current_version.as_deref().unwrap_or(&self.i18n.t("ui.not_selected"))));
                if ui.button(self.i18n.t("ui.refresh")).clicked() {
                    pending_action = Some(Action::Refresh);
                }
                
                ui.separator();
                let mut selected_lang = self.config.language.clone();
                let lang_text = match selected_lang.as_str() {
                    "vi" => "🌐 Tiếng Việt",
                    _ => "🌐 English",
                };
                
                egui::ComboBox::from_id_salt("lang_dropdown")
                    .selected_text(lang_text)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut selected_lang, "en".to_string(), "English").changed() {
                            pending_action = Some(Action::ChangeLanguage("en".to_string()));
                        }
                        if ui.selectable_value(&mut selected_lang, "vi".to_string(), "Tiếng Việt").changed() {
                            pending_action = Some(Action::ChangeLanguage("vi".to_string()));
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.small(self.i18n.t("ui.saved_at").replace("{}", &self.config.base_dir.display().to_string()));
                if ui.button(self.i18n.t("ui.change_location")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        pending_action = Some(Action::MoveStorage(path));
                    }
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(self.i18n.t("ui.search"));
                ui.text_edit_singleline(&mut self.search_query);
                ui.checkbox(&mut self.lts_only, self.i18n.t("ui.lts_only"));
            });

            ui.separator();

            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, self.i18n.t("ui.error_prefix").replace("{}", err));
            }

            ui.label(&self.status_msg);

            if self.is_loading {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    if let Some((copied, total)) = self.move_progress {
                        let progress = copied as f32 / total as f32;
                        ui.add(egui::ProgressBar::new(progress).text(format!("{} / {} files", copied, total)));
                    }
                });
            }

            ui.separator();

            let mut groups: BTreeMap<i32, Vec<&NodeVersion>> = BTreeMap::new();
            for v in &self.versions {
                if self.lts_only && !v.is_lts() { continue; }
                if !self.search_query.is_empty() && !v.version.contains(&self.search_query) { continue; }

                let major = v.version.trim_start_matches('v').split('.').next()
                    .and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                groups.entry(major).or_default().push(v);
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (major, versions) in groups.iter().rev() {
                        egui::CollapsingHeader::new(format!("Node v{} ({})", major, versions.len()))
                            .default_open(*major >= 20) 
                            .show(ui, |ui| {
                                egui::Grid::new(format!("grid_{}", major))
                                    .num_columns(5)
                                    .spacing([10.0, 8.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(self.i18n.t("ui.version_col"));
                                        ui.label(self.i18n.t("ui.type_col"));
                                        ui.label(self.i18n.t("ui.status_col"));
                                        ui.label(self.i18n.t("ui.shared_col"));
                                        ui.label(self.i18n.t("ui.action_col"));
                                        ui.end_row();

                                        for v in versions {
                                            let is_installed = self.config.installed_versions.contains(&v.version);
                                            let is_current = self.config.current_version.as_ref() == Some(&v.version);

                                            if is_current {
                                                ui.colored_label(egui::Color32::from_rgb(255, 215, 0), format!("● {}", v.version));
                                            } else {
                                                ui.label(format!("  {}", v.version));
                                            }

                                            if v.is_lts() {
                                                ui.colored_label(egui::Color32::GREEN, "LTS");
                                            } else {
                                                ui.label("-");
                                            }

                                            if is_installed {
                                                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), self.i18n.t("ui.downloaded"));
                                            } else {
                                                ui.label(self.i18n.t("ui.not_downloaded"));
                                            }

                                            if is_installed {
                                                let mut use_shared = self.config.version_configs.get(&v.version).cloned().unwrap_or(false);
                                                if ui.checkbox(&mut use_shared, "").changed() {
                                                    self.config.version_configs.insert(v.version.clone(), use_shared);
                                                    if is_current {
                                                        pending_action = Some(Action::UpdateConfig);
                                                    } else {
                                                        if let Err(e) = self.config.save() {
                                                            self.error = Some(self.i18n.t("status.saving_config_error").replace("{}", &e.to_string()));
                                                        }
                                                    }
                                                }
                                            } else {
                                                ui.label("");
                                            }

                                            ui.horizontal(|ui| {
                                                if is_current {
                                                    ui.label(self.i18n.t("ui.in_use"));
                                                    if ui.button(self.i18n.t("ui.unuse_btn")).clicked() {
                                                        pending_action = Some(Action::Unuse);
                                                    }
                                                } else if is_installed {
                                                    if ui.button(self.i18n.t("ui.use_btn")).clicked() {
                                                        pending_action = Some(Action::Switch(v.version.clone()));
                                                    }
                                                    if ui.button(self.i18n.t("ui.delete_btn")).clicked() {
                                                        pending_action = Some(Action::Uninstall(v.version.clone()));
                                                    }
                                                } else {
                                                    if !self.is_loading && ui.button(self.i18n.t("ui.install_btn")).clicked() {
                                                        pending_action = Some(Action::Install(v.version.clone()));
                                                    }
                                                }
                                            });
                                            ui.end_row();
                                        }
                                    });
                            });
                    }
                });
        });

        if let Some(action) = pending_action {
            self.handle_action(action);
        }

        if self.is_loading {
            ui.ctx().request_repaint();
        }
    }
}

// Các hàm bổ trợ Logic nội bộ
fn copy_dir_all(
    src: &std::path::Path, 
    dst: &std::path::Path, 
    tx: &Sender<AppMessage>, 
    last_update: &mut Instant,
    copied: &mut usize,
    total: usize,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_all(&path, &dst.join(entry.file_name()), tx, last_update, copied, total)?;
        } else {
            *copied += 1;
            if last_update.elapsed().as_millis() > 100 {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let _ = tx.send(AppMessage::MoveProgress(*copied, total));
                let _ = tx.send(AppMessage::StatusUpdate(format!("Copying: {}", file_name)));
                *last_update = Instant::now();
            }
            std::fs::copy(&path, dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn count_files(path: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                count += count_files(&entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

fn is_dir_empty(path: &std::path::Path) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(path) {
        entries.next().is_none()
    } else {
        false
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Nhúng thẳng font Inter (hỗ trợ Tiếng Việt cực tốt) vào file exe/binary để ứng dụng chạy mượt trên mọi OS
    let font_data = include_bytes!("../assets/Inter-Regular.ttf");

    fonts.font_data.insert(
        "inter_font".to_owned(),
        egui::FontData::from_static(font_data).into(),
    );
    
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "inter_font".to_owned());
    }
    
    ctx.set_fonts(fonts);
}
