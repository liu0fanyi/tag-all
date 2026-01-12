use tauri::{AppHandle, command, Runtime};
use tauri_plugin_dialog::DialogExt;

#[command]
pub async fn pick_folder<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let result = app.dialog().file().blocking_pick_folder();
        match result {
            Some(path) => Ok(Some(path.to_string())),
            None => Ok(None),
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // Not supported/implemented on mobile for now
        Ok(None)
    }
}
