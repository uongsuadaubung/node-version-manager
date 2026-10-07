#[path = "compat/directories.rs"]
pub mod directories;

#[path = "compat/anyhow.rs"]
#[macro_use]
pub mod anyhow;

#[cfg(windows)]
#[path = "compat/winreg.rs"]
pub mod winreg;

#[cfg(windows)]
#[path = "compat/windows.rs"]
pub mod windows;

pub mod app;
pub mod config;
pub mod downloader;
pub mod env_manager;
pub mod i18n;
pub mod utils;
pub mod version_service;
