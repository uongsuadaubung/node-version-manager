use std::path::{Path, PathBuf};

pub struct UserDirs {
    home: PathBuf,
}

impl UserDirs {
    pub fn new() -> Option<Self> {
        let home = if cfg!(windows) {
            std::env::var_os("USERPROFILE")
                .or_else(|| std::env::var_os("HOME"))
                .map(PathBuf::from)?
        } else {
            std::env::var_os("HOME").map(PathBuf::from)?
        };

        Some(Self { home })
    }

    pub fn home_dir(&self) -> &Path {
        &self.home
    }
}

