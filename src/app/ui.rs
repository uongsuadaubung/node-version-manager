use eframe::egui;
use std::collections::BTreeMap;

use crate::version_service::NodeVersion;

use super::{Action, AppMessage, NvmApp};

impl eframe::App for NvmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Xử lý messages từ background threads
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::Fetch(fetch_msg) => match fetch_msg {
                    crate::app::FetchMsg::Success(v) => {
                        self.versions = v;
                        self.is_loading = false;
                    }
                    crate::app::FetchMsg::Error(e) => {
                        self.error = Some(e);
                        self.is_loading = false;
                    }
                },
                AppMessage::Download(download_msg) => match download_msg {
                    crate::app::DownloadMsg::Finished(v) => {
                        if !self.config.installed_versions.contains(&v) {
                            self.config.installed_versions.push(v);
                        }
                        if let Err(e) = self.config.save() {
                            self.error = Some(
                                self.i18n
                                    .t("status.saving_config_error")
                                    .replace("{}", &e.to_string()),
                            );
                        }
                        self.status_msg = self.i18n.t("status.install_success");
                        self.is_loading = false;
                        self.download_progress = None;
                    }
                    crate::app::DownloadMsg::Error(e) => {
                        self.error = Some(e);
                        self.is_loading = false;
                        self.download_progress = None;
                    }
                    crate::app::DownloadMsg::Progress(downloaded, total) => {
                        self.download_progress = Some((downloaded, total));
                    }
                },
                AppMessage::Storage(storage_msg) => match storage_msg {
                    crate::app::StorageMsg::Finished(new_path) => {
                        let old_base = self.config.base_dir.clone();
                        self.config.base_dir = new_path;
                        self.update_config_and_env(Some(&old_base));
                        self.status_msg = self.i18n.t("status.move_success");
                        self.is_loading = false;
                        self.move_progress = None;
                    }
                    crate::app::StorageMsg::Error(e) => {
                        self.error = Some(e);
                        self.is_loading = false;
                        self.move_progress = None;
                    }
                    crate::app::StorageMsg::Progress(copied, total) => {
                        self.move_progress = Some((copied, total));
                    }
                },
                AppMessage::General(crate::app::GeneralMsg::StatusUpdate(s)) => {
                    self.status_msg = s;
                }
            }
        }

        let mut pending_action = None;

        ui.vertical(|ui| {
            ui.heading("Node Version Manager");

            ui.horizontal(|ui| {
                ui.label(
                    self.i18n.t("ui.current_version").replace(
                        "{}",
                        self.config
                            .current_version
                            .as_deref()
                            .unwrap_or(&self.i18n.t("ui.not_selected")),
                    ),
                );
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
                        if ui
                            .selectable_value(&mut selected_lang, "en".to_string(), "English")
                            .changed()
                        {
                            pending_action = Some(Action::ChangeLanguage("en".to_string()));
                        }
                        if ui
                            .selectable_value(&mut selected_lang, "vi".to_string(), "Tiếng Việt")
                            .changed()
                        {
                            pending_action = Some(Action::ChangeLanguage("vi".to_string()));
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.small(
                    self.i18n
                        .t("ui.saved_at")
                        .replace("{}", &self.config.base_dir.display().to_string()),
                );
                if ui.button(self.i18n.t("ui.change_location")).clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_folder()
                {
                    pending_action = Some(Action::MoveStorage(path));
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
                ui.colored_label(
                    egui::Color32::RED,
                    self.i18n.t("ui.error_prefix").replace("{}", err),
                );
            }

            ui.label(&self.status_msg);

            if self.is_loading {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    if let Some((downloaded, total)) = self.download_progress {
                        if total > 0 {
                            let progress = downloaded as f32 / total as f32;
                            ui.add(egui::ProgressBar::new(progress).text(format!(
                                "📥 {} / {} MB",
                                downloaded / 1_048_576,
                                total / 1_048_576
                            )));
                        } else {
                            ui.add(egui::ProgressBar::new(0.0).text("📥 Downloading..."));
                        }
                    } else if let Some((copied, total)) = self.move_progress {
                        let progress = copied as f32 / total as f32;
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .text(format!("📦 {} / {} files", copied, total)),
                        );
                    }
                });
            }

            ui.separator();

            let mut groups: BTreeMap<i32, Vec<&NodeVersion>> = BTreeMap::new();
            for v in &self.versions {
                if self.lts_only && !v.is_lts() {
                    continue;
                }
                if !self.search_query.is_empty() && !v.version.contains(&self.search_query) {
                    continue;
                }

                let major = v
                    .version
                    .trim_start_matches('v')
                    .split('.')
                    .next()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                groups.entry(major).or_default().push(v);
            }

            let max_major = groups.keys().next_back().copied();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (major, versions) in groups.iter().rev() {
                        egui::CollapsingHeader::new(format!(
                            "Node v{} ({})",
                            major,
                            versions.len()
                        ))
                        .default_open(Some(*major) == max_major)
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
                                        let is_installed =
                                            self.config.installed_versions.contains(&v.version);
                                        let is_current = self.config.current_version.as_ref()
                                            == Some(&v.version);

                                        if is_current {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 215, 0),
                                                format!("● {}", v.version),
                                            );
                                        } else {
                                            ui.label(format!("  {}", v.version));
                                        }

                                        if let Some(name) = v.lts_name() {
                                            ui.colored_label(
                                                egui::Color32::GREEN,
                                                format!("LTS ({})", name),
                                            );
                                        } else {
                                            ui.label("-");
                                        }

                                        if is_installed {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(100, 200, 255),
                                                self.i18n.t("ui.downloaded"),
                                            );
                                        } else {
                                            ui.label(self.i18n.t("ui.not_downloaded"));
                                        }

                                        if is_installed {
                                            let mut use_shared = self
                                                .config
                                                .version_configs
                                                .get(&v.version)
                                                .cloned()
                                                .unwrap_or(false);
                                            if ui.checkbox(&mut use_shared, "").changed() {
                                                self.config
                                                    .version_configs
                                                    .insert(v.version.clone(), use_shared);
                                                if is_current {
                                                    pending_action = Some(Action::UpdateConfig);
                                                } else if let Err(e) = self.config.save() {
                                                    self.error = Some(
                                                        self.i18n
                                                            .t("status.saving_config_error")
                                                            .replace("{}", &e.to_string()),
                                                    );
                                                }
                                            }
                                        } else {
                                            ui.label("");
                                        }

                                        ui.horizontal(|ui| {
                                            if is_current {
                                                ui.label(self.i18n.t("ui.in_use"));
                                                if ui.button(self.i18n.t("ui.unuse_btn")).clicked()
                                                {
                                                    pending_action = Some(Action::Unuse);
                                                }
                                            } else if is_installed {
                                                if ui.button(self.i18n.t("ui.use_btn")).clicked() {
                                                    pending_action =
                                                        Some(Action::Switch(v.version.clone()));
                                                }
                                                if ui.button(self.i18n.t("ui.delete_btn")).clicked()
                                                {
                                                    pending_action =
                                                        Some(Action::Uninstall(v.version.clone()));
                                                }
                                            } else if !self.is_loading
                                                && ui
                                                    .button(self.i18n.t("ui.install_btn"))
                                                    .clicked()
                                            {
                                                pending_action =
                                                    Some(Action::Install(v.version.clone()));
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

/// Font được nhúng thẳng vào binary để chạy mượt trên mọi OS
pub(super) fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_data = include_bytes!("../../assets/Inter-Regular.ttf");

    fonts.font_data.insert(
        "inter_font".to_owned(),
        egui::FontData::from_static(font_data).into(),
    );

    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "inter_font".to_owned());
    }

    ctx.set_fonts(fonts);
}
