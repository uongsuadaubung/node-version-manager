use std::fs;
use std::io;
use std::path::PathBuf;
use zip::ZipArchive;

pub fn extract_archive(
    archive_path: &PathBuf,
    dest_base: &PathBuf,
    _dir_name: &str,
) -> anyhow::Result<PathBuf> {
    let mut root = PathBuf::new();
    let zip_file = fs::File::open(archive_path)?;
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
    Ok(root)
}
