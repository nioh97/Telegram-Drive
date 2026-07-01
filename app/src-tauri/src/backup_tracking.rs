use crate::db::DbConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadedMedia {
    pub id: String,
    pub media_store_id: Option<String>,
    pub uri: Option<String>,
    pub path: Option<String>,
    pub size: Option<i64>,
    pub date_taken: Option<i64>,
    pub sha256: Option<String>,
    pub telegram_message_id: Option<i64>,
    pub upload_time: i64,
}

pub fn is_media_uploaded(db: &DbConnection, media_store_id: &Option<String>, sha256: &Option<String>) -> Result<bool, String> {
    let conn = db.lock().unwrap();
    
    // Check by media_store_id or sha256 to handle edge cases
    if let Some(ms_id) = media_store_id {
        let mut stmt = conn.prepare("SELECT 1 FROM uploaded_media WHERE media_store_id = ?").map_err(|e| e.to_string())?;
        stmt.bind((1, ms_id.as_str())).map_err(|e| e.to_string())?;
        if stmt.next().map_err(|e| e.to_string())? == sqlite::State::Row {
            return Ok(true);
        }
    }
    
    if let Some(hash) = sha256 {
        let mut stmt = conn.prepare("SELECT 1 FROM uploaded_media WHERE sha256 = ?").map_err(|e| e.to_string())?;
        stmt.bind((1, hash.as_str())).map_err(|e| e.to_string())?;
        if stmt.next().map_err(|e| e.to_string())? == sqlite::State::Row {
            return Ok(true);
        }
    }
    
    Ok(false)
}

pub fn track_uploaded_media(db: &DbConnection, media: UploadedMedia) -> Result<(), String> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO uploaded_media (id, media_store_id, uri, path, size, date_taken, sha256, telegram_message_id, upload_time) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .map_err(|e| e.to_string())?;

    stmt.bind((1, media.id.as_str())).map_err(|e| e.to_string())?;
    
    if let Some(ms) = media.media_store_id { stmt.bind((2, ms.as_str())).map_err(|e| e.to_string())?; }
    if let Some(uri) = media.uri { stmt.bind((3, uri.as_str())).map_err(|e| e.to_string())?; }
    if let Some(path) = media.path { stmt.bind((4, path.as_str())).map_err(|e| e.to_string())?; }
    if let Some(size) = media.size { stmt.bind((5, size)).map_err(|e| e.to_string())?; }
    if let Some(dt) = media.date_taken { stmt.bind((6, dt)).map_err(|e| e.to_string())?; }
    if let Some(hash) = media.sha256 { stmt.bind((7, hash.as_str())).map_err(|e| e.to_string())?; }
    if let Some(msg_id) = media.telegram_message_id { stmt.bind((8, msg_id)).map_err(|e| e.to_string())?; }
    
    stmt.bind((9, media.upload_time)).map_err(|e| e.to_string())?;

    stmt.next().map_err(|e| e.to_string())?;
    Ok(())
}
