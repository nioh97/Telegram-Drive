pub fn start_foreground_service() {
    // Replaced by WorkManager in BackupWorker.kt
}

pub fn stop_foreground_service() {
    // Replaced by WorkManager in BackupWorker.kt
}

#[tauri::command]
pub fn cmd_start_foreground_service() {
    start_foreground_service();
}

#[tauri::command]
pub fn cmd_stop_foreground_service() {
    stop_foreground_service();
}
