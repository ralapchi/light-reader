use serde::{Deserialize, Serialize};

use crate::domain::recent_book_item::RecentBookItem;
use crate::storage::paths;

const RECENT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct RecentFile {
    version: u32,
    items: Vec<RecentBookItem>,
}

pub fn load() -> Vec<RecentBookItem> {
    let path = paths::recent_books_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => {
            let file: RecentFile = serde_json::from_str(&data).unwrap_or_else(|_| RecentFile {
                version: RECENT_VERSION,
                items: Vec::new(),
            });
            file.items
        }
        Err(_) => Vec::new(),
    }
}

pub fn save(items: &[RecentBookItem]) -> Result<(), String> {
    let path = paths::recent_books_path();
    let file = RecentFile {
        version: RECENT_VERSION,
        items: items.to_vec(),
    };
    let data = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}
