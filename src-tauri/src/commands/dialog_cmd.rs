use tauri::{AppHandle, command, Runtime};
use tauri_plugin_dialog::DialogExt;

#[command]
pub async fn pick_folder<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    let result = app.dialog().file().blocking_pick_folder();
    match result {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}
