use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::time::Instant;

use rayon::prelude::*;

use super::{AppMessage, GeneralMsg, StorageMsg};
use crate::i18n::I18n;
use crate::utils;

pub fn move_task(
    path: PathBuf,
    old_base: PathBuf,
    installed_versions: Vec<String>,
    lang: String,
    tx: Sender<AppMessage>,
) -> anyhow::Result<()> {
    let i18n = I18n::new(&lang);

    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| {
            anyhow::anyhow!(
                i18n.t("status.create_new_dir_error")
                    .replace("{}", &e.to_string())
            )
        })?;
    }

    let num_threads = std::thread::available_parallelism()
        .map(|n| (n.get() * 4).min(64))
        .unwrap_or(16);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()?;

    let versions_from_dir = old_base.join("versions");
    let mut total_files = 0;
    tx.send(AppMessage::General(GeneralMsg::StatusUpdate(i18n.t("status.counting_files")))).ok();
    for v in &installed_versions {
        total_files += count_files(&versions_from_dir.join(utils::get_version_dir_name(v)));
    }
    total_files += count_files(&old_base.join("modules"));
    tx.send(AppMessage::General(GeneralMsg::StatusUpdate(
        i18n.t("status.moving_files")
            .replace("{}", &total_files.to_string()),
    ))).ok();

    let versions_to_dir = path.join("versions");
    let copied_count = Arc::new(AtomicUsize::new(0));

    if versions_from_dir.exists() {
        std::fs::create_dir_all(&versions_to_dir).map_err(|e| {
            anyhow::anyhow!(
                i18n.t("status.create_versions_dir_error")
                    .replace("{}", &e.to_string())
            )
        })?;

        pool.install(|| {
            installed_versions
                .par_iter()
                .try_for_each(|v| -> anyhow::Result<()> {
                    let dir_name = utils::get_version_dir_name(v);
                    let from = versions_from_dir.join(&dir_name);
                    let to = versions_to_dir.join(&dir_name);
                    if !from.exists() {
                        return Ok(());
                    }

                    if std::fs::rename(&from, &to).is_ok() {
                        let n = count_files(&to);
                        let new_n = copied_count.fetch_add(n, Ordering::Relaxed) + n;
                        tx.send(AppMessage::Storage(StorageMsg::Progress(
                            new_n.min(total_files),
                            total_files,
                        )))
                        .ok();
                        return Ok(());
                    }

                    copy_dir_parallel(&from, &to, &tx, &copied_count, total_files, &i18n).map_err(
                        |e| {
                            anyhow::anyhow!(
                                i18n.t("status.copy_error")
                                    .replacen("{}", &dir_name, 1)
                                    .replacen("{}", &e.to_string(), 1)
                            )
                        },
                    )?;
                    tx.send(AppMessage::General(GeneralMsg::StatusUpdate(
                        i18n.t("status.cleaning_up").replace("{}", &dir_name),
                    )))
                    .ok();
                    remove_dir_parallel(&from, &tx, &i18n);
                    Ok(())
                })
        })?;

        if is_dir_empty(&versions_from_dir) {
            std::fs::remove_dir(&versions_from_dir).ok();
        }
    }

    let modules_from = old_base.join("modules");
    let modules_to = path.join("modules");
    if modules_from.exists() {
        if std::fs::rename(&modules_from, &modules_to).is_err() {
            pool.install(|| {
                copy_dir_parallel(
                    &modules_from,
                    &modules_to,
                    &tx,
                    &copied_count,
                    total_files,
                    &i18n,
                )
            })
            .map_err(|e| {
                anyhow::anyhow!(
                    i18n.t("status.copy_modules_error")
                        .replace("{}", &e.to_string())
                )
            })?;
            tx.send(AppMessage::General(GeneralMsg::StatusUpdate(
                i18n.t("status.cleaning_up_modules"),
            )))
            .ok();
            remove_dir_parallel(&modules_from, &tx, &i18n);
        } else {
            let n = count_files(&modules_to);
            let new_n = copied_count.fetch_add(n, Ordering::Relaxed) + n;
            tx.send(AppMessage::Storage(StorageMsg::Progress(
                new_n.min(total_files),
                total_files,
            )))
            .ok();
        }
        if modules_from.exists() && is_dir_empty(&modules_from) {
            std::fs::remove_dir(&modules_from).ok();
        }
    }

    tx.send(AppMessage::Storage(StorageMsg::Finished(path))).ok();
    Ok(())
}

pub fn count_files(path: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                count += count_files(&entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

pub fn is_dir_empty(path: &Path) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(path) {
        entries.next().is_none()
    } else {
        false
    }
}

/// Copy cross-device: walk song song, pre-create dirs, copy song song.
fn copy_dir_parallel(
    src: &Path,
    dst: &Path,
    tx: &Sender<AppMessage>,
    shared_copied: &Arc<AtomicUsize>,
    total: usize,
    i18n: &I18n,
) -> anyhow::Result<()> {
    let pairs = walk_parallel(src, dst)?;

    let dir_name = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    tx.send(AppMessage::General(GeneralMsg::StatusUpdate(
        i18n.t("status.copying_dir")
            .replacen("{}", &dir_name, 1)
            .replacen("{}", &pairs.len().to_string(), 1),
    )))
    .ok();

    let unique_dirs: HashSet<&Path> = pairs
        .iter()
        .filter_map(|(_, dst_f)| dst_f.parent())
        .collect();
    unique_dirs
        .par_iter()
        .try_for_each(|d| std::fs::create_dir_all(d).map_err(anyhow::Error::from))?;

    let start = Instant::now();
    let last_update_ms = AtomicU64::new(0);

    pairs
        .par_iter()
        .try_for_each(|(src_f, dst_f)| -> anyhow::Result<()> {
            std::fs::copy(src_f, dst_f)?;
            shared_copied.fetch_add(1, Ordering::Relaxed);

            let now_ms = start.elapsed().as_millis() as u64;
            let last = last_update_ms.load(Ordering::Relaxed);
            if now_ms.saturating_sub(last) >= 100
                && last_update_ms
                    .compare_exchange(last, now_ms, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                let n = shared_copied.load(Ordering::Relaxed);
                tx.send(AppMessage::Storage(StorageMsg::Progress(n.min(total), total))).ok();
            }
            Ok(())
        })?;

    let n = shared_copied.load(Ordering::Relaxed);
    tx.send(AppMessage::Storage(StorageMsg::Progress(n.min(total), total))).ok();

    Ok(())
}

/// Xóa thư mục song song: xóa files bằng rayon, sau đó xóa dirs từ deep → shallow.
fn remove_dir_parallel(path: &Path, tx: &Sender<AppMessage>, i18n: &I18n) {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Err(e) = collect_for_removal(path, &mut files, &mut dirs) {
        tx.send(AppMessage::General(GeneralMsg::StatusUpdate(
            i18n.t("status.cleanup_walk_error")
                .replace("{}", &e.to_string()),
        )))
        .ok();
        return;
    }

    let file_errors = AtomicUsize::new(0);
    files.par_iter().for_each(|f| {
        if std::fs::remove_file(f).is_err() {
            file_errors.fetch_add(1, Ordering::Relaxed);
        }
    });

    let mut dir_errors: usize = 0;
    dirs.sort_by_key(|d| Reverse(d.components().count()));
    for d in &dirs {
        if std::fs::remove_dir(d).is_err() {
            dir_errors += 1;
        }
    }
    if std::fs::remove_dir(path).is_err() {
        dir_errors += 1;
    }

    let total_failed = file_errors.load(Ordering::Relaxed) + dir_errors;
    if total_failed > 0 {
        tx.send(AppMessage::General(GeneralMsg::StatusUpdate(
            i18n.t("status.cleanup_failed")
                .replace("{}", &total_failed.to_string()),
        )))
        .ok();
    }
}

/// Thu thập tất cả files và dirs trong cây thư mục để xóa.
fn collect_for_removal(
    path: &Path,
    files: &mut Vec<PathBuf>,
    dirs: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(path)?.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_for_removal(&p, files, dirs)?;
            dirs.push(p);
        } else {
            files.push(p);
        }
    }
    Ok(())
}

/// Walk song song: tại mỗi level, các subdir được walk song song bằng rayon.
fn walk_parallel(src: &Path, dst: &Path) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
    let entries: Vec<_> = std::fs::read_dir(src)?.flatten().collect();
    let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.path().is_dir());

    let mut pairs: Vec<(PathBuf, PathBuf)> = files
        .into_iter()
        .map(|e| (e.path(), dst.join(e.file_name())))
        .collect();

    let sub: Vec<anyhow::Result<Vec<(PathBuf, PathBuf)>>> = dirs
        .par_iter()
        .map(|d| walk_parallel(&d.path(), &dst.join(d.file_name())))
        .collect();

    for result in sub {
        pairs.extend(result?);
    }

    Ok(pairs)
}
