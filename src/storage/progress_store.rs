use log::warn;
use serde::{Deserialize, Serialize};

use crate::domain::reading_progress::ReadingProgress;
use crate::storage::paths;

const PROGRESS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct ProgressFile {
    version: u32,
    progress: ReadingProgress,
}

/// Borrowed wrapper to avoid cloning progress for serialization.
#[derive(Serialize)]
struct ProgressFileRef<'a> {
    version: u32,
    progress: &'a ReadingProgress,
}

pub fn load(book_id: &str) -> Option<ReadingProgress> {
    let path = paths::progress_path(book_id);
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<ProgressFile>(&data) {
            Ok(file) => {
                // T12: 版本检查
                if file.version != PROGRESS_VERSION {
                    warn!(
                        "进度文件版本不匹配 (book_id={}): 期望 {}，实际 {}，尝试兼容读取",
                        book_id, PROGRESS_VERSION, file.version
                    );
                }
                Some(file.progress)
            }
            Err(e) => {
                warn!(
                    "阅读进度文件解析失败 (book_id={}): {}，回退到章节开头",
                    book_id, e
                );
                None
            }
        },
        Err(e) => {
            warn!(
                "阅读进度文件读取失败 (book_id={}): {}，回退到章节开头",
                book_id, e
            );
            None
        }
    }
}

pub fn save(book_id: &str, progress: &ReadingProgress) -> Result<(), String> {
    let path = paths::progress_path(book_id);
    let file = ProgressFileRef {
        version: PROGRESS_VERSION,
        progress,
    };
    crate::storage::util::write_json_atomic(&path, &file).map_err(|e| e.to_string())
}
