use std::fs;
use std::io;
use std::path::PathBuf;
use zip::ZipArchive;
use reqwest::blocking::Client;

pub fn download_and_extract(version: &str, dest_base: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Ví dụ: v20.11.0 -> https://nodejs.org/dist/v20.11.0/node-v20.11.0-win-x64.zip
    let url = format!("https://nodejs.org/dist/{}/node-{}-win-x64.zip", version, version);
    let client = Client::new();
    let mut response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()).into());
    }
    
    if !dest_base.exists() {
        fs::create_dir_all(dest_base)?;
    }
    
    let zip_path = dest_base.join(format!("{}.zip", version));
    let mut file = fs::File::create(&zip_path)?;
    io::copy(&mut response, &mut file)?;
    
    // Giải nén
    let zip_file = fs::File::open(&zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;
    
    let mut extracted_root = PathBuf::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_base.join(path),
            None => continue,
        };

        // Lấy tên thư mục gốc bên trong zip (thường là node-vXX.XX.XX-win-x64)
        if i == 0 {
            if let Some(first_part) = outpath.components().nth(dest_base.components().count()) {
                extracted_root = dest_base.join(first_part.as_os_str());
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
    
    // Xóa file zip tạm
    let _ = fs::remove_file(zip_path);
    
    Ok(extracted_root)
}
