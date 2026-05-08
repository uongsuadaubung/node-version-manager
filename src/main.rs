#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod version_service;
mod env_manager;
mod downloader;
mod config;
mod app;

use eframe::egui;
use app::NvmApp;

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
