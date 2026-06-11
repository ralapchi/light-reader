use std::path::PathBuf;
use std::sync::OnceLock;

const APP_DIR_NAME: &str = "light-reader";

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn app_data_dir() -> PathBuf {
    APP_DATA_DIR
        .get_or_init(|| {
            dirs::data_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
                .unwrap_or_else(|| PathBuf::from("."))
                .join(APP_DIR_NAME)
        })
        .clone()
}

pub fn ensure_dirs() -> std::io::Result<()> {
    let base = app_data_dir();
    std::fs::create_dir_all(&base)?;
    std::fs::create_dir_all(base.join("progress"))?;
    std::fs::create_dir_all(base.join("bookmarks"))?;
    std::fs::create_dir_all(base.join("cache/covers"))?;
    std::fs::create_dir_all(base.join("cache/images"))?;
    std::fs::create_dir_all(base.join("cache/tts"))?;
    Ok(())
}

pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn progress_path(book_id: &str) -> PathBuf {
    app_data_dir()
        .join("progress")
        .join(format!("{}.json", book_id))
}

pub fn bookmarks_path(book_id: &str) -> PathBuf {
    app_data_dir()
        .join("bookmarks")
        .join(format!("{}.json", book_id))
}

pub fn library_index_path() -> PathBuf {
    app_data_dir().join("library_index.json")
}

pub fn cover_cache_path(book_id: &str, ext: &str) -> PathBuf {
    app_data_dir()
        .join("cache/covers")
        .join(format!("{}.{}", book_id, ext))
}

/// Probe the cover cache directory for any matching image extension.
pub fn find_cover_by_extensions(book_id: &str) -> Option<PathBuf> {
    let cover_dir = app_data_dir().join("cache/covers");
    for ext in &["png", "jpg", "jpeg", "webp", "gif", "svg"] {
        let p = cover_dir.join(format!("{}.{}", book_id, ext));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn image_cache_path(book_id: &str, asset_id: &str, ext: &str) -> PathBuf {
    app_data_dir()
        .join("cache/images")
        .join(book_id)
        .join(format!("{}.{}", asset_id, ext))
}

pub fn tts_cache_dir() -> PathBuf {
    app_data_dir().join("cache/tts")
}
