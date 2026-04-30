use serde::{Deserialize, Serialize};

use crate::domain::bookmark::Bookmark;
use crate::storage::paths;

const BOOKMARKS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct BookmarksFile {
    version: u32,
    book_id: String,
    items: Vec<Bookmark>,
}

pub fn load(book_id: &str) -> Vec<Bookmark> {
    let path = paths::bookmarks_path(book_id);
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => {
            let file: BookmarksFile = serde_json::from_str(&data).unwrap_or_else(|_| BookmarksFile {
                version: BOOKMARKS_VERSION,
                book_id: book_id.to_string(),
                items: Vec::new(),
            });
            file.items
        }
        Err(_) => Vec::new(),
    }
}

pub fn save(book_id: &str, bookmarks: &[Bookmark]) -> Result<(), String> {
    let path = paths::bookmarks_path(book_id);
    let file = BookmarksFile {
        version: BOOKMARKS_VERSION,
        book_id: book_id.to_string(),
        items: bookmarks.to_vec(),
    };
    let data = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}
