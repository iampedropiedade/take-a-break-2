use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::error::AppError;

pub fn images_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir()?.join("images");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Copies a user-picked image into the app's own images directory under a
/// generated filename, decoupling the stored break from the original path
/// (which may live on removable media or be renamed/moved later).
pub fn copy_image_into_store(app: &AppHandle, src_path: &Path) -> Result<String, AppError> {
    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let dest = images_dir(app)?.join(&filename);
    fs::copy(src_path, &dest)?;
    Ok(filename)
}

pub fn resolve_image_path(app: &AppHandle, filename: &str) -> Result<PathBuf, AppError> {
    Ok(images_dir(app)?.join(filename))
}
