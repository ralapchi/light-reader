use log::warn;

use crate::domain::library_item::LibraryIndex;
use crate::storage::paths;

const LIBRARY_VERSION: u32 = 1;

pub fn load() -> LibraryIndex {
    let path = paths::library_index_path();
    if !path.exists() {
        return LibraryIndex::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<LibraryIndex>(&data) {
            Ok(index) => {
                if index.version != LIBRARY_VERSION {
                    warn!(
                        "书库索引版本不匹配: 期望 {}，实际 {}，尝试兼容读取",
                        LIBRARY_VERSION, index.version
                    );
                }
                index
            }
            Err(e) => {
                warn!("书库索引解析失败: {}，返回默认值", e);
                LibraryIndex::default()
            }
        },
        Err(e) => {
            warn!("书库索引读取失败: {}，返回默认值", e);
            LibraryIndex::default()
        }
    }
}

pub fn save(index: &LibraryIndex) -> Result<(), std::io::Error> {
    let path = paths::library_index_path();
    let mut index = index.clone();
    index.version = LIBRARY_VERSION;
    crate::storage::util::write_json_atomic(&path, &index)
}
