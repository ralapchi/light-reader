use log::warn;
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
        Ok(data) => match serde_json::from_str::<RecentFile>(&data) {
            Ok(file) => {
                // T12: 版本检查
                if file.version != RECENT_VERSION {
                    warn!(
                        "最近阅读文件版本不匹配: 期望 {}，实际 {}，尝试兼容读取",
                        RECENT_VERSION, file.version
                    );
                }
                // T13: 跳过损坏项
                file.items
            }
            Err(e) => {
                warn!("最近阅读文件解析失败: {}，返回空列表", e);
                Vec::new()
            }
        },
        Err(e) => {
            warn!("最近阅读文件读取失败: {}，返回空列表", e);
            Vec::new()
        }
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
