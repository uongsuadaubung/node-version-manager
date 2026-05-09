use std::fs;
use std::io;
use std::path::PathBuf;
use reqwest::blocking::Client;
use crate::utils;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::extract_archive;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix::extract_archive;

pub fn download_and_extract(version: &str, dest_base: &PathBuf) -> anyhow::Result<PathBuf> {
    #[cfg(windows)]
    let extension = "zip";
    #[cfg(unix)]
    let extension = "tar.gz";

    let dir_name = utils::get_version_dir_name(version);
    let url = format!("https://nodejs.org/dist/{}/{}.{}", version, dir_name, extension);
    
    let client = Client::new();
    let mut response = client.get(url).send()?;
    
    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }
    
    if !dest_base.exists() {
        fs::create_dir_all(dest_base)?;
    }
    
    let archive_path = dest_base.join(format!("{}.{}", version, extension));
    let mut file = fs::File::create(&archive_path)?;
    io::copy(&mut response, &mut file)?;
    
    let extracted_root = extract_archive(&archive_path, dest_base, &dir_name)?;
    
    // Xóa file nén tạm
    let _ = fs::remove_file(archive_path);
    
    Ok(extracted_root)
}
