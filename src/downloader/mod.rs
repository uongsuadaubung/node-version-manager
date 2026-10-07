use crate::anyhow;
use crate::app::{AppMessage, DownloadMsg};
use crate::utils;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Instant;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::extract_archive;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix::extract_archive;

pub fn download_and_extract(
    version: &str,
    dest_base: &PathBuf,
    tx: &Sender<AppMessage>,
) -> anyhow::Result<PathBuf> {
    #[cfg(windows)]
    let extension = "zip";
    #[cfg(unix)]
    let extension = "tar.gz";

    let dir_name = utils::get_version_dir_name(version);
    let url = format!(
        "https://nodejs.org/dist/{}/{}.{}",
        version, dir_name, extension
    );

    let response = ureq::get(&url)
        .set("User-Agent", "nvm-rust-gui")
        .call()?;

    if response.status() != 200 {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let total_bytes: u64 = response
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut reader = response.into_reader();

    if !dest_base.exists() {
        fs::create_dir_all(dest_base)?;
    }

    let archive_path = dest_base.join(format!("{}.{}", version, extension));
    let mut file = fs::File::create(&archive_path)?;

    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65536]; // 64KB chunks
    let mut last_update = Instant::now();

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;

        if last_update.elapsed().as_millis() > 100 {
            tx.send(AppMessage::Download(DownloadMsg::Progress(downloaded, total_bytes)))
                .ok();
            last_update = Instant::now();
        }
    }
    // Gửi lần cuối để cập nhật 100%
    tx.send(AppMessage::Download(DownloadMsg::Progress(downloaded, total_bytes)))
        .ok();

    let extracted_root = extract_archive(&archive_path, dest_base, &dir_name)?;

    // Xóa file nén tạm
    fs::remove_file(archive_path).ok();

    Ok(extracted_root)
}
