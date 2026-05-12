use flate2::read::GzDecoder;
use std::fs;
use std::path::PathBuf;
use tar::Archive;

pub fn extract_archive(
    archive_path: &PathBuf,
    dest_base: &PathBuf,
    dir_name: &str,
) -> anyhow::Result<PathBuf> {
    let tar_gz = fs::File::open(archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(dest_base)?;
    Ok(dest_base.join(dir_name))
}
