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
            Ok(file) => {
                // T12: 版本检查
                if file.version != BOOKMARKS_VERSION {
                    warn!(
                        "书签文件版本不匹配 (book_id={}): 期望 {}，实际 {}，尝试兼容读取",
                        book_id, BOOKMARKS_VERSION, file.version
                    );
                }
                file.items
            }
            Err(e) => {
                warn!(
                    "书签文件解析失败 (book_id={}): {}，返回空书签列表",
                    book_id, e
                );
                Vec::new()
            }
        },
        Err(e) => {
            warn!(
                "书签文件读取失败 (book_id={}): {}，返回空书签列表",
                book_id, e
            );
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
    crate::storage::util::write_json_atomic(&path, &file).map_err(|e| e.to_string())
}

pub fn load_all() -> Vec<Bookmark> {
    let dir = paths::app_data_dir().join("bookmarks");
    let mut all = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return all,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<BookmarksFile>(&data) {
                Ok(file) => all.extend(file.items),
                Err(e) => warn!("书签文件解析失败 ({:?}): {}", path, e),
            },
            Err(e) => warn!("书签文件读取失败 ({:?}): {}", path, e),
        }
    }
    all
}
