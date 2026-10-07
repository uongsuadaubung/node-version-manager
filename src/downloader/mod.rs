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

    let total_bytes = crate::fetch::get_content_length(&url);

    let mut child = crate::fetch::stream_download(&url)?;

    let mut reader = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to pipe curl output"))?;

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

    drop(file);
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("Download failed with curl exit code: {:?}", status.code());
    }

    // Gửi lần cuối để cập nhật 100%
    tx.send(AppMessage::Download(DownloadMsg::Progress(downloaded, total_bytes)))
        .ok();

    let extracted_root = extract_archive(&archive_path, dest_base, &dir_name)?;

    // Xóa file nén tạm
    fs::remove_file(archive_path).ok();

    Ok(extracted_root)
}
