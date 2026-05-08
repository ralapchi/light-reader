use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::book_assets::BookAssets;
use crate::domain::book_format::BookFormat;
use crate::domain::book_load_info::BookLoadInfo;
use crate::domain::book_metadata::BookMetadata;
use crate::domain::chapter::Chapter;
use crate::domain::toc_item::TocItem;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub source_path: PathBuf,
    pub format: BookFormat,
    pub metadata: BookMetadata,
    pub toc: Vec<TocItem>,
    pub chapters: Vec<Chapter>,
    pub assets: BookAssets,
    pub load_info: BookLoadInfo,
}

/// Generate a stable identifier from a file path using canonicalized path hashing.
pub fn stable_book_id(path: &str) -> String {
    let normalized = std::fs::canonicalize(path)
        .ok()
        .and_then(|resolved| resolved.to_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| path.to_string());
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("book-{:016x}", hasher.finish())
}
