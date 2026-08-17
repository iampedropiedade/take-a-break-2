use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::error::AppError;
use crate::persistence::images;

#[tauri::command]
pub async fn pick_image(app: AppHandle) -> Result<Option<String>, AppError> {
    let file = app
        .dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .blocking_pick_file();

    let Some(file) = file else {
        return Ok(None);
    };
    let path = file
        .into_path()
        .map_err(|e| AppError::Message(e.to_string()))?;
    let filename = images::copy_image_into_store(&app, &path)?;
    Ok(Some(filename))
}

#[tauri::command]
pub fn get_image_path(app: AppHandle, filename: String) -> Result<String, AppError> {
    let path = images::resolve_image_path(&app, &filename)?;
    Ok(path.to_string_lossy().to_string())
}
