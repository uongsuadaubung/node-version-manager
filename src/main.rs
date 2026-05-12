#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod downloader;
mod env_manager;
mod i18n;
mod utils;
mod version_service;

use app::NvmApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Khắc phục lỗi tương thích Vulkan trên Wayland/Linux bằng cách mặc định dùng OpenGL
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WGPU_BACKEND").is_err() {
            unsafe { std::env::set_var("WGPU_BACKEND", "gl") };
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_min_inner_size([400.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Node Version Manager GUI",
        options,
        Box::new(|cc| Ok(Box::new(NvmApp::new(cc)))),
    )
}
