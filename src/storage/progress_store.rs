use serde::{Deserialize, Serialize};

use crate::domain::reading_progress::ReadingProgress;
use crate::storage::paths;

const PROGRESS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct ProgressFile {
    version: u32,
    progress: ReadingProgress,
}

pub fn load(book_id: &str) -> Option<ReadingProgress> {
    let path = paths::progress_path(book_id);
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => {
            let file: ProgressFile = serde_json::from_str(&data).ok()?;
            Some(file.progress)
        }
        Err(_) => None,
    }
}

pub fn save(book_id: &str, progress: &ReadingProgress) -> Result<(), String> {
    let path = paths::progress_path(book_id);
    let file = ProgressFile {
        version: PROGRESS_VERSION,
        progress: progress.clone(),
    };
    let data = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}
