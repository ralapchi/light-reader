use log::warn;
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
        Ok(data) => match serde_json::from_str::<BookmarksFile>(&data) {
            Ok(file) => file.items,
            Err(e) => {
                warn!("书签文件解析失败 (book_id={}): {}", book_id, e);
                Vec::new()
            }
        },
        Err(e) => {
            warn!("书签文件读取失败 (book_id={}): {}", book_id, e);
            Vec::new()
        }
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
