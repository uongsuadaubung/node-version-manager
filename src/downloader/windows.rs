use crate::anyhow;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn extract_archive(
    archive_path: &Path,
    dest_base: &Path,
    dir_name: &str,
) -> anyhow::Result<PathBuf> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new("tar");
    cmd.creation_flags(CREATE_NO_WINDOW);

    let status = cmd
        .arg("-xf")
        .arg(archive_path)
        .arg("-C")
        .arg(dest_base)
        .status()?;

    if !status.success() {
        anyhow::bail!("Extraction failed with exit code: {:?}", status.code());
    }

    Ok(dest_base.join(dir_name))
}
