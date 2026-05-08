#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Ẩn console window khi chạy bản release

mod version_service;
mod env_manager;
mod downloader;
mod config;

use eframe::egui;
use version_service::{fetch_node_versions, NodeVersion};
use config::AppConfig;
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_min_inner_size([400.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Node Version Manager (Rust)",
        options,
        Box::new(|cc| Ok(Box::new(NvmApp::new(cc)))),
    )
}

struct NvmApp {
    config: AppConfig,
    versions: Vec<NodeVersion>,
    is_loading: bool,
    lts_only: bool,
    search_query: String,
    move_progress: Option<(usize, usize)>,
    error: Option<String>,
    status_msg: String,
    tx: Sender<AppMessage>,
    rx: Receiver<AppMessage>,
}

enum AppMessage {
    VersionsFetched(Vec<NodeVersion>),
    FetchError(String),
    InstallFinished(String),
    InstallError(String),
    MoveFinished(std::path::PathBuf),
    MoveError(String),
    StatusUpdate(String),
    MoveProgress(usize, usize), // (đã copy, tổng số)
}

impl NvmApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        setup_custom_fonts(&cc.egui_ctx);

        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();

        // Tải danh sách phiên bản khi khởi động
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
            lts_only: true, // Mặc định chỉ hiện LTS cho gọn
            search_query: "".to_string(),
            move_progress: None,
            error: None,
            status_msg: "Đang sẵn sàng...".to_string(),
            tx,
            rx,
        }
    }

    fn update_config_and_env(&mut self) {
        let _ = self.config.save();
        
        // Nếu có phiên bản đang dùng, cập nhật lại PATH/npmrc theo cấu hình RIÊNG của nó
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
                
                let _ = env_manager::update_user_path(&version_path, modules_dir.as_ref());
                let _ = env_manager::update_npmrc(&self.config.modules_dir(), use_shared);
            }
        }
    }
}

fn copy_dir_all(
    src: &std::path::Path, 
    dst: &std::path::Path, 
    tx: &Sender<AppMessage>, 
    last_update: &mut Instant,
    copied: &mut usize,
    total: usize,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_all(&path, &dst.join(entry.file_name()), tx, last_update, copied, total)?;
        } else {
            *copied += 1;
            // Throttle: Chỉ gửi update tối đa 100ms một lần
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
    // Đọc trực tiếp font Arial từ Windows để hỗ trợ tiếng Việt mà không làm nặng file exe
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

impl eframe::App for NvmApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        // Nhận tin nhắn từ các luồng khác
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
                    self.config.base_dir = new_path;
                    self.update_config_and_env();
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
                if ui.button("🔄 Làm mới").on_hover_text("Tải lại danh sách phiên bản từ server").clicked() {
                    pending_action = Some(Action::Refresh);
                }
                
                ui.separator();
                
                ui.small(format!("Lưu tại: {}", self.config.base_dir.display()));
                if ui.button("📁 Đổi nơi lưu").on_hover_text("Di chuyển toàn bộ Node versions sang thư mục khác").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        pending_action = Some(Action::MoveStorage(path));
                    }
                }
            });

            ui.separator();

            // Toolbar: Tìm kiếm và Lọc
            ui.horizontal(|ui| {
                ui.label("Tìm kiếm:");
                ui.text_edit_singleline(&mut self.search_query);
                ui.checkbox(&mut self.lts_only, "Chỉ bản LTS");
            });

            ui.separator();

            if let Some(err) = &self.error {
                ui.colored_label(eframe::egui::Color32::RED, format!("Lỗi: {}", err));
            }

            ui.label(&self.status_msg);

            if self.is_loading {
                ui.horizontal(|ui| {
                    ui.add(eframe::egui::Spinner::new());
                    if let Some((copied, total)) = self.move_progress {
                        let progress = copied as f32 / total as f32;
                        ui.add(eframe::egui::ProgressBar::new(progress).text(format!("{} / {} files", copied, total)));
                    }
                });
            }

            ui.separator();

            // Logic Nhóm theo Major Version
            use std::collections::BTreeMap;
            let mut groups: BTreeMap<i32, Vec<&NodeVersion>> = BTreeMap::new();
            
            for v in &self.versions {
                if self.lts_only && !v.is_lts() { continue; }
                if !self.search_query.is_empty() && !v.version.contains(&self.search_query) { continue; }

                let major = v.version.trim_start_matches('v').split('.').next()
                    .and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                groups.entry(major).or_default().push(v);
            }

            eframe::egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (major, versions) in groups.iter().rev() {
                        let header_text = format!("Node v{} ({})", major, versions.len());
                        egui::CollapsingHeader::new(header_text)
                            .default_open(*major >= 20) 
                            .show(ui, |ui| {
                                egui::Grid::new(format!("grid_{}", major))
                                    .num_columns(5)
                                    .spacing([10.0, 8.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        // Header của bảng (nhỏ gọn)
                                        ui.label("Phiên bản");
                                        ui.label("Kiểu");
                                        ui.label("Trạng thái");
                                        ui.label("Dùng chung");
                                        ui.label("Hành động");
                                        ui.end_row();

                                        for v in versions {
                                            let is_installed = self.config.installed_versions.contains(&v.version);
                                            let is_current = self.config.current_version.as_ref() == Some(&v.version);

                                            // 1. Phiên bản
                                            if is_current {
                                                ui.colored_label(egui::Color32::from_rgb(255, 215, 0), format!("● {}", v.version));
                                            } else {
                                                ui.label(format!("  {}", v.version));
                                            }

                                            // 2. Kiểu (LTS)
                                            if v.is_lts() {
                                                ui.colored_label(egui::Color32::GREEN, "LTS");
                                            } else {
                                                ui.label("-");
                                            }

                                            // 3. Trạng thái
                                            if is_installed {
                                                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), "✔ Đã tải");
                                            } else {
                                                ui.label("○ Chưa tải");
                                            }

                                            // 4. Dùng chung (Shared)
                                            if is_installed {
                                                let mut use_shared = self.config.version_configs.get(&v.version).cloned().unwrap_or(false);
                                                if ui.checkbox(&mut use_shared, "").on_hover_text("Dùng chung thư mục global modules").changed() {
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

                                            // 5. Hành động
                                            ui.horizontal(|ui| {
                                                if is_current {
                                                    ui.label("✅ Đang dùng");
                                                } else if is_installed {
                                                    if ui.button("🚀 Sử dụng").on_hover_text("Kích hoạt phiên bản Node này cho hệ thống").clicked() {
                                                        pending_action = Some(Action::Switch(v.version.clone()));
                                                    }
                                                    if ui.button("🗑 Xóa").on_hover_text("Gỡ bỏ phiên bản này khỏi máy tính").clicked() {
                                                        pending_action = Some(Action::Uninstall(v.version.clone()));
                                                    }
                                                } else {
                                                    if !self.is_loading && ui.button("📥 Cài đặt").on_hover_text("Tải và cài đặt phiên bản Node này").clicked() {
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

        // Xử lý các hành động
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
                    self.update_config_and_env();
                }
                Action::Switch(v) => {
                    self.config.current_version = Some(v.clone());
                    self.update_config_and_env();
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

                        // Đếm tổng số file trước
                        let mut total_files = 0;
                        let versions_from_dir = old_base.join("versions");
                        for v in &installed_versions {
                            total_files += count_files(&versions_from_dir.join(format!("node-{}-win-x64", v)));
                        }
                        total_files += count_files(&old_base.join("modules"));

                        let _ = tx.send(AppMessage::StatusUpdate(format!("Tổng cộng {} files. Bắt đầu di chuyển...", total_files)));

                        // 1. Di chuyển từng phiên bản Node cụ thể
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
                                        let _ = copy_dir_all(&from, &to, &tx, &mut last_update, &mut copied_count, total_files);
                                        let _ = std::fs::remove_dir_all(&from);
                                    } else {
                                        // Nếu rename thành công, ta coi như đã "copy" xong các file trong đó (tăng counter)
                                        copied_count += count_files(&to);
                                        let _ = tx.send(AppMessage::MoveProgress(copied_count, total_files));
                                    }
                                }
                            }
                            // Dọn dẹp thư mục versions cũ nếu rỗng
                            if is_dir_empty(&versions_from_dir) {
                                let _ = std::fs::remove_dir(&versions_from_dir);
                            }
                        }

                        // 2. Di chuyển thư mục modules (nếu có)
                        let modules_from = old_base.join("modules");
                        let modules_to = path.join("modules");
                        if modules_from.exists() {
                            if std::fs::rename(&modules_from, &modules_to).is_err() {
                                let _ = copy_dir_all(&modules_from, &modules_to, &tx, &mut last_update, &mut copied_count, total_files);
                                let _ = std::fs::remove_dir_all(&modules_from);
                            } else {
                                copied_count += count_files(&modules_to);
                                let _ = tx.send(AppMessage::MoveProgress(copied_count, total_files));
                            }
                            // Dọn dẹp thư mục modules cũ nếu rỗng (thường rename đã xóa rồi, nhưng phòng hờ fallback)
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

enum Action {
    Refresh,
    UpdateConfig,
    Switch(String),
    Install(String),
    Uninstall(String),
    MoveStorage(std::path::PathBuf),
}
