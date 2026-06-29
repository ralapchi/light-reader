use std::sync::Mutex;

use log::info;
use tauri::Manager;

use crate::domain::library_item::LibraryIndex;
use crate::storage::traits::DatabaseBackend;
use crate::tauri_api::commands::{ProgressState, DirtyProgressState};

pub struct AppData {
    pub db: Box<dyn DatabaseBackend>,
    pub library_index: LibraryIndex,
}

pub fn init() -> AppData {
    info!("Tauri 模式启动");

    let settings = crate::storage::settings_store::load();
    let data_dir = crate::storage::paths::app_data_dir();
    let db_config = settings.database.clone().unwrap_or_default();
    let db: Box<dyn DatabaseBackend> = crate::storage::factory::create_backend(&db_config, &data_dir)
        .expect("数据库初始化失败");

    let library_index = match db.books().list_all() {
        Ok(items) if !items.is_empty() => {
            let last_selected = db.books().get_last_selected().ok().flatten();
            LibraryIndex {
                version: 1,
                items,
                last_selected_book_id: last_selected,
            }
        }
        _ => crate::storage::library_store::load(),
    };

    AppData { db, library_index }
}

pub fn on_window_close(window: &tauri::Window) {
    flush_progress(window);
    flush_library(window);
}

fn flush_progress(window: &tauri::Window) {
    let Some(progress) = window.try_state::<ProgressState>() else { return };
    let Some(dirty) = window.try_state::<DirtyProgressState>() else { return };
    let Some(db) = window.try_state::<Box<dyn DatabaseBackend>>() else { return };

    if let Err(e) = crate::tauri_api::commands::flush_dirty_progress_to_db(
        progress.inner(),
        dirty.inner(),
        db.inner(),
    ) {
        log::warn!("退出时保存阅读进度失败: {}", e);
    }
}

fn flush_library(window: &tauri::Window) {
    let Some(guard) = window.try_state::<Mutex<LibraryIndex>>() else { return };
    let Ok(index) = guard.lock() else { return };

    if let Some(db) = window.try_state::<Box<dyn DatabaseBackend>>() {
        for item in &index.items {
            if let Err(e) = db.books().upsert(item) {
                log::warn!("退出时保存书籍到数据库失败: {}", e);
            }
        }
        if let Some(ref id) = index.last_selected_book_id {
            let _ = db.books().set_last_selected(id);
        }
    }

    crate::services::library_service_impl::LibraryServiceImpl::save_index(&index);
}
