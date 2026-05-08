use std::fs;
use std::io;
use std::path::PathBuf;
use reqwest::blocking::Client;
use crate::utils;

#[cfg(windows)]
use zip::ZipArchive;

#[cfg(unix)]
use flate2::read::GzDecoder;
#[cfg(unix)]
use tar::Archive;

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
    
    #[cfg(windows)]
    let extracted_root = {
        let mut root = PathBuf::new();
        let zip_file = fs::File::open(&archive_path)?;
        let mut archive = ZipArchive::new(zip_file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest_base.join(path),
                None => continue,
            };

            if i == 0 {
                if let Some(first_part) = outpath.components().nth(dest_base.components().count()) {
                    root = dest_base.join(first_part.as_os_str());
                }
            }

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        root
    };

    #[cfg(unix)]
    let extracted_root = {
        let tar_gz = fs::File::open(&archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        
        archive.unpack(dest_base)?;
        dest_base.join(&dir_name)
    };
    
    // Xóa file nén tạm
    let _ = fs::remove_file(archive_path);
    
    Ok(extracted_root)
}
