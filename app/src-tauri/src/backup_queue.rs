use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BackupStatus {
    Pending,
    Uploading,
    Uploaded,
    Failed(String),
    Retry,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupJob {
    pub id: String,
    pub path: String,
    pub uri: Option<String>,
    pub media_store_id: Option<String>,
    pub status: BackupStatus,
}

pub struct BackupQueueState {
    pub queue: Arc<Mutex<VecDeque<BackupJob>>>,
    pub is_paused: Arc<std::sync::atomic::AtomicBool>,
}

impl BackupQueueState {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            is_paused: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

#[tauri::command]
pub async fn cmd_enqueue_backup(
    state: tauri::State<'_, BackupQueueState>,
    job: BackupJob,
) -> Result<(), String> {
    let mut q = state.queue.lock().await;
    q.push_back(job);
    Ok(())
}

#[tauri::command]
pub async fn cmd_get_backup_queue(
    state: tauri::State<'_, BackupQueueState>,
) -> Result<Vec<BackupJob>, String> {
    let q = state.queue.lock().await;
    Ok(q.iter().cloned().collect())
}

#[tauri::command]
pub async fn cmd_pause_backup(
    state: tauri::State<'_, BackupQueueState>,
) -> Result<(), String> {
    state.is_paused.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn cmd_resume_backup(
    state: tauri::State<'_, BackupQueueState>,
) -> Result<(), String> {
    state.is_paused.store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
