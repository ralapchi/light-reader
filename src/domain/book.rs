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
