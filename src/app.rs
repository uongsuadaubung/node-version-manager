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

        Self {
            config: AppConfig::load(),
            versions: Vec::new(),
            is_loading: true,
            lts_only: true,
            search_query: "".to_string(),
            move_progress: None,
            error: None,
            status_msg: "Đang sẵn sàng...".to_string(),
            tx,
            rx,
        }
    }

    pub fn update_config_and_env(&mut self, old_base_dir: Option<&PathBuf>) {
        let _ = self.config.save();
        
        if let Some(ref version) = self.config.current_version {
            let version_path = self.config.versions_dir().join(format!("node-{}-win-x64", version));
            if version_path.exists() {
                let use_shared = self.config.version_configs.get(version).cloned().unwrap_or(false);
                
                let modules_dir = if use_shared {
                    let m_dir = self.config.modules_dir();
                    if !m_dir.exists() {
                        let _ = std::fs::create_dir_all(&m_dir);
                    }
                    Some(m_dir)
                } else {
                    None
                };
                
                let _ = env_manager::update_user_path(&version_path, modules_dir.as_ref(), &self.config.base_dir, old_base_dir);
                let _ = env_manager::update_npmrc(&self.config.modules_dir(), use_shared);
            }
        }
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
                    let _ = self.config.save();
                    self.status_msg = "Cài đặt thành công!".to_string();
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
                    self.status_msg = "Di chuyển dữ liệu thành công!".to_string();
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
                ui.label(format!("Đang dùng: {}", self.config.current_version.as_deref().unwrap_or("Chưa chọn")));
                if ui.button("🔄 Làm mới").clicked() {
                    pending_action = Some(Action::Refresh);
                }
                ui.separator();
                ui.small(format!("Lưu tại: {}", self.config.base_dir.display()));
                if ui.button("📁 Đổi nơi lưu").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        pending_action = Some(Action::MoveStorage(path));
                    }
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Tìm kiếm:");
                ui.text_edit_singleline(&mut self.search_query);
                ui.checkbox(&mut self.lts_only, "Chỉ bản LTS");
            });

            ui.separator();

            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Lỗi: {}", err));
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
                                        ui.label("Phiên bản");
                                        ui.label("Kiểu");
                                        ui.label("Trạng thái");
                                        ui.label("Dùng chung");
                                        ui.label("Hành động");
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
                                                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), "✔ Đã tải");
                                            } else {
                                                ui.label("○ Chưa tải");
                                            }

                                            if is_installed {
                                                let mut use_shared = self.config.version_configs.get(&v.version).cloned().unwrap_or(false);
                                                if ui.checkbox(&mut use_shared, "").changed() {
                                                    self.config.version_configs.insert(v.version.clone(), use_shared);
                                                    if is_current {
                                                        pending_action = Some(Action::UpdateConfig);
                                                    } else {
                                                        let _ = self.config.save();
                                                    }
                                                }
                                            } else {
                                                ui.label("");
                                            }

                                            ui.horizontal(|ui| {
                                                if is_current {
                                                    ui.label("✅ Đang dùng");
                                                } else if is_installed {
                                                    if ui.button("🚀 Sử dụng").clicked() {
                                                        pending_action = Some(Action::Switch(v.version.clone()));
                                                    }
                                                    if ui.button("🗑 Xóa").clicked() {
                                                        pending_action = Some(Action::Uninstall(v.version.clone()));
                                                    }
                                                } else {
                                                    if !self.is_loading && ui.button("📥 Cài đặt").clicked() {
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
            match action {
                Action::Refresh => {
                    self.is_loading = true;
                    let tx = self.tx.clone();
                    thread::spawn(move || {
                        match version_service::fetch_node_versions() {
                            Ok(v) => { let _ = tx.send(AppMessage::VersionsFetched(v)); },
                            Err(e) => { let _ = tx.send(AppMessage::FetchError(e.to_string())); }
                        }
                    });
                }
                Action::UpdateConfig => {
                    self.update_config_and_env(None);
                }
                Action::Switch(v) => {
                    self.config.current_version = Some(v.clone());
                    self.update_config_and_env(None);
                    self.status_msg = format!("Đã chuyển sang {}", v);
                }
                Action::Install(v) => {
                    self.is_loading = true;
                    self.status_msg = format!("Đang cài đặt {}...", v);
                    let tx = self.tx.clone();
                    let base_dir = self.config.versions_dir();
                    thread::spawn(move || {
                        match downloader::download_and_extract(&v, &base_dir) {
                            Ok(_) => { let _ = tx.send(AppMessage::InstallFinished(v)); },
                            Err(e) => { let _ = tx.send(AppMessage::InstallError(e.to_string())); }
                        }
                    });
                }
                Action::Uninstall(v) => {
                    let version_path = self.config.versions_dir().join(format!("node-{}-win-x64", v));
                    if version_path.exists() {
                        if let Err(e) = std::fs::remove_dir_all(&version_path) {
                            self.error = Some(format!("Không thể xóa thư mục: {}", e));
                        } else {
                            self.config.installed_versions.retain(|iv| iv != &v);
                            self.config.version_configs.remove(&v);
                            let _ = self.config.save();
                            self.status_msg = format!("Đã xóa phiên bản {}", v);
                        }
                    }
                }
                Action::MoveStorage(path) => {
                    self.is_loading = true;
                    self.status_msg = "Đang chuẩn bị di chuyển...".to_string();
                    let tx = self.tx.clone();
                    let old_base = self.config.base_dir.clone();
                    let installed_versions = self.config.installed_versions.clone();
                    
                    thread::spawn(move || {
                        if !path.exists() {
                            let _ = std::fs::create_dir_all(&path);
                        }

                        let mut total_files = 0;
                        let versions_from_dir = old_base.join("versions");
                        for v in &installed_versions {
                            total_files += count_files(&versions_from_dir.join(format!("node-{}-win-x64", v)));
                        }
                        total_files += count_files(&old_base.join("modules"));

                        let _ = tx.send(AppMessage::StatusUpdate(format!("Tổng cộng {} files. Bắt đầu di chuyển...", total_files)));

                        let mut copied_count = 0;
                        let mut last_update = Instant::now();
                        let versions_to_dir = path.join("versions");
                        
                        if versions_from_dir.exists() {
                            let _ = std::fs::create_dir_all(&versions_to_dir);
                            for v in installed_versions {
                                let dir_name = format!("node-{}-win-x64", v);
                                let from = versions_from_dir.join(&dir_name);
                                let to = versions_to_dir.join(&dir_name);
                                if from.exists() {
                                    if std::fs::rename(&from, &to).is_err() {
                                        if let Err(e) = copy_dir_all(&from, &to, &tx, &mut last_update, &mut copied_count, total_files) {
                                            let _ = tx.send(AppMessage::MoveError(format!("Lỗi copy {}: {}", dir_name, e)));
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
                                        let _ = tx.send(AppMessage::MoveError(format!("Lỗi copy modules: {}", e)));
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
        }

        if self.is_loading {
            ui.ctx().request_repaint();
        }
    }
}

// Các hàm bổ trợ UI/Logic
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
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\arial.ttf") {
        fonts.font_data.insert(
            "arial_sys".to_owned(),
            egui::FontData::from_owned(font_data).into(),
        );
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "arial_sys".to_owned());
    }
    ctx.set_fonts(fonts);
}
