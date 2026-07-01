use tauri::{AppHandle, Manager, Emitter};
use serde::{Deserialize, Serialize};
use crate::db::DbConnection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupPreferences {
    pub enabled: bool,
    pub target_chat_id: Option<i64>,
    pub target_folder_id: Option<i64>,
    pub network_type: String, // "wifi", "cellular", "charging"
    pub sources_photos: bool,
    pub sources_videos: bool,
    pub sources_screenshots: bool,
    pub sources_downloads: bool,
    pub sources_whatsapp: bool,
}

impl Default for BackupPreferences {
    fn default() -> Self {
        Self {
            enabled: false,
            target_chat_id: None,
            target_folder_id: None,
            network_type: "wifi".to_string(),
            sources_photos: true,
            sources_videos: false,
            sources_screenshots: false,
            sources_downloads: false,
            sources_whatsapp: false,
        }
    }
}

pub fn get_preferences(db: &DbConnection) -> Result<BackupPreferences, String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    let mut cursor = conn
        .prepare("SELECT value FROM backup_settings WHERE key = 'preferences'")
        .map_err(|e| e.to_string())?
        .into_iter();

    if let Some(Ok(row)) = cursor.next() {
        let value: String = row.read::<&str, _>(0).to_string();
        if let Ok(prefs) = serde_json::from_str(&value) {
            return Ok(prefs);
        }
    }
    Ok(BackupPreferences::default())
}

#[tauri::command]
pub fn cmd_get_backup_preferences(app_handle: AppHandle) -> Result<BackupPreferences, String> {
    let db = app_handle.state::<DbConnection>();
    get_preferences(&db)
}

#[tauri::command]
pub fn cmd_set_backup_preferences(prefs: BackupPreferences, app_handle: AppHandle) -> Result<(), String> {
    let db = app_handle.state::<DbConnection>();
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    
    let json = serde_json::to_string(&prefs).map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare("INSERT OR REPLACE INTO backup_settings (key, value) VALUES ('preferences', ?)")
        .map_err(|e| e.to_string())?;
    
    stmt.bind((1, json.as_str())).map_err(|e| e.to_string())?;
    stmt.next().map_err(|e| e.to_string())?;
    
    // Tell Android Worker to update its schedule if enabled state changed
    #[cfg(target_os = "android")]
    {
        // TODO: Call Kotlin method to register/unregister WorkManager
    }
    
    Ok(())
}

pub fn init_backup_worker(app_handle: AppHandle, db: DbConnection) {
    let bridge_path = {
        #[cfg(target_os = "android")]
        {
            // Android: Kotlin writes to Context.filesDir which is /data/user/0/<pkg>/files/
            // app_data_dir() gives us /data/user/0/<pkg>/ so we must append "files"
            let mut dir = app_handle.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("/data/data/com.cameronamer.telegramdrive"));
            dir.push("files");
            // Ensure the directory exists
            let _ = std::fs::create_dir_all(&dir);
            dir.join("backup_bridge.jsonl")
        }
        #[cfg(not(target_os = "android"))]
        {
            let dir = app_handle.path().app_data_dir().unwrap_or_default();
            dir.join("backup_bridge.jsonl")
        }
    };

    println!("[Backup] Starting bridge poller. Path: {:?}", bridge_path);
    log::info!("[Backup] Starting bridge poller. Path: {:?}", bridge_path);

    // Let's write an empty string to it immediately so Kotlin doesn't fail if the file didn't exist,
    // though Kotlin's appendText creates it if missing.
    let _ = std::fs::OpenOptions::new().create(true).write(true).open(&bridge_path);

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let content = match std::fs::read_to_string(&bridge_path) {
                Ok(c) => c,
                Err(e) => {
                    // println!("[Backup] Poller error reading bridge file: {}", e);
                    continue;
                }
            };

            if content.trim().is_empty() {
                continue;
            }

            println!("[Backup] Bridge file has content! Length: {}", content.len());
            log::info!("[Backup] Bridge file has content! Length: {}", content.len());

            // Truncate immediately
            let _ = std::fs::write(&bridge_path, "");

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }

                let media_array: Vec<serde_json::Value> = match serde_json::from_str(line) {
                    Ok(arr) => arr,
                    Err(e) => {
                        println!("[Backup] Failed to parse bridge line: {}", e);
                        log::error!("[Backup] Failed to parse bridge line: {}", e);
                        continue;
                    }
                };

                println!("[Backup] Received {} media items from bridge file.", media_array.len());
                log::info!("[Backup] Received {} media items from bridge file.", media_array.len());

                if let Ok(conn) = db.lock() {
                    for item in &media_array {
                        let id = item.get("media_store_id").and_then(|v| v.as_str()).unwrap_or("");
                        let path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        let mime = item.get("mime_type").and_then(|v| v.as_str()).unwrap_or("");
                        let size = item.get("size").and_then(|v| v.as_i64()).unwrap_or(0);
                        let date_added = item.get("date_added").and_then(|v| v.as_i64()).unwrap_or(0);

                        let query = "INSERT OR IGNORE INTO upload_queue (id, media_store_id, uri, path, mime_type, size, date_added, priority, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
                        if let Ok(mut stmt) = conn.prepare(query) {
                            let queue_id = format!("q-{}", id);
                            let priority = if mime.starts_with("image/") { 1 } else { 2 };
                            let _ = stmt.bind((1, queue_id.as_str()));
                            let _ = stmt.bind((2, id));
                            let _ = stmt.bind((3, "")); // uri
                            let _ = stmt.bind((4, path));
                            let _ = stmt.bind((5, mime));
                            let _ = stmt.bind((6, size));
                            let _ = stmt.bind((7, date_added));
                            let _ = stmt.bind((8, priority as i64));
                            let _ = stmt.bind((9, "Pending"));
                            let _ = stmt.next();
                        }

                        println!("[Backup] Queued: {} ({})", path, mime);
                        log::info!("[Backup] Queued: {} ({})", path, mime);
                    }
                }
            }

            let db_clone = db.clone();
            let app_clone = app_handle.clone();
            let _ = process_upload_queue(db_clone, app_clone).await;

            app_handle.emit("backup_queue_updated", ()).unwrap_or(());
        }
    });
}

pub async fn process_upload_queue(db: DbConnection, app_handle: AppHandle) -> Result<(), String> {
    let items_to_process = {
        let conn = db.lock().map_err(|_| "db lock failed")?;
        let stmt = conn.prepare("SELECT id, path FROM upload_queue WHERE status = 'Pending' ORDER BY priority ASC LIMIT 5").unwrap();
        let mut batch = Vec::new();
        for row in stmt.into_iter().filter_map(|r| r.ok()) {
            batch.push((
                row.read::<&str, _>(0).to_string(),
                row.read::<&str, _>(1).to_string(),
            ));
        }
        batch
    };

    if items_to_process.is_empty() {
        return Ok(());
    }

    log::info!("[Backup] Processing {} pending uploads...", items_to_process.len());

    // Get backup preferences (folder_id to upload to)
    let prefs = get_preferences(&db)?;
    let target_folder_id = prefs.target_folder_id;

    for (id, path) in items_to_process {
        log::info!("[Backup] Upload started: {}", path);
        
        {
            let conn = db.lock().unwrap();
            let _ = conn.execute(format!("UPDATE upload_queue SET status = 'Uploading' WHERE id = '{}'", id));
        }
        
        // Fetch states from Tauri manager
        let state = app_handle.state::<crate::commands::TelegramState>();
        let bw_state = app_handle.state::<std::sync::Arc<crate::bandwidth::BandwidthManager>>();
        let net_config = app_handle.state::<std::sync::Arc<crate::vpn_optimizer::NetworkConfig>>();
        
        // Perform actual Grammers upload using the fs command
        let res = crate::commands::fs::cmd_upload_file(
            path.clone(),
            target_folder_id,
            Some(id.clone()),
            app_handle.clone(),
            state,
            bw_state,
            net_config
        ).await;
        
        {
            let conn = db.lock().unwrap();
            if let Err(e) = res {
                log::error!("[Backup] Upload failed for {}: {}", path, e);
                let _ = conn.execute(format!("UPDATE upload_queue SET status = 'Failed', retry_count = retry_count + 1 WHERE id = '{}'", id));
            } else {
                log::info!("[Backup] Upload completed: {}", path);
                let _ = conn.execute(format!("UPDATE upload_queue SET status = 'Uploaded' WHERE id = '{}'", id));
            }
        }
    }
    
    Ok(())
}

