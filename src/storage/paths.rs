use std::path::PathBuf;

const APP_DIR_NAME: &str = "reader-demo";

fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME)
}

pub fn ensure_dirs() -> std::io::Result<()> {
    let base = app_data_dir();
    std::fs::create_dir_all(&base)?;
    std::fs::create_dir_all(base.join("progress"))?;
    std::fs::create_dir_all(base.join("bookmarks"))?;
    std::fs::create_dir_all(base.join("cache/covers"))?;
    std::fs::create_dir_all(base.join("cache/images"))?;
    Ok(())
}

pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn recent_books_path() -> PathBuf {
    app_data_dir().join("recent_books.json")
}

pub fn progress_path(book_id: &str) -> PathBuf {
    app_data_dir().join("progress").join(format!("{}.json", book_id))
}

pub fn bookmarks_path(book_id: &str) -> PathBuf {
    app_data_dir().join("bookmarks").join(format!("{}.json", book_id))
}

pub fn library_index_path() -> PathBuf {
    app_data_dir().join("library_index.json")
}

pub fn cover_cache_path(book_id: &str, ext: &str) -> PathBuf {
    app_data_dir().join("cache/covers").join(format!("{}.{}", book_id, ext))
}

pub fn image_cache_path(asset_id: &str, ext: &str) -> PathBuf {
    app_data_dir().join("cache/images").join(format!("{}.{}", asset_id, ext))
}
