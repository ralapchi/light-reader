/*!
EPUB 阅读器主入口
*/

use std::io::Write;
use std::sync::Mutex;

use log::info;

mod domain;
mod parser;
mod services;
mod storage;
mod tauri_api;
mod tts;

/// Writes to two `Write` targets simultaneously.
struct TeeWriter<A: Write, B: Write> {
    a: Mutex<A>,
    b: Mutex<B>,
}

impl<A: Write, B: Write> Write for TeeWriter<A, B> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut a) = self.a.lock() { a.write_all(buf).ok(); }
        if let Ok(mut b) = self.b.lock() { b.write_all(buf).ok(); }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if let Ok(mut a) = self.a.lock() { a.flush().ok(); }
        if let Ok(mut b) = self.b.lock() { b.flush().ok(); }
        Ok(())
    }
}

fn init_logging() {
    let log_dir = storage::paths::app_data_dir().join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let level_var = std::env::var("RUST_LOG")
        .or_else(|_| std::env::var("READER_LOG"))
        .unwrap_or_else(|_| "info".to_string());

    let log_path = log_dir.join("reader.log");
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => {
            let tee = TeeWriter {
                a: Mutex::new(std::io::stderr()),
                b: Mutex::new(file),
            };
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&level_var))
                .target(env_logger::Target::Pipe(Box::new(tee)))
                .init();
            info!("日志启动 (级别={}, 文件={})", level_var, log_path.display());
        }
        Err(e) => {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&level_var))
                .target(env_logger::Target::Stderr)
                .init();
            eprintln!("日志文件创建失败 ({}), 回退到 stderr: {}", log_path.display(), e);
        }
    }
}

fn main() {
    init_logging();

    use tauri_api::commands::*;

    info!("Tauri 模式启动");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(ReaderState::new()))
        .manage(Mutex::new(TtsSession::new()))
        .invoke_handler(tauri::generate_handler![
            // Library
            library_list,
            library_import,
            library_open,
            library_remove,
            library_remove_batch,
            library_search,
            library_repair_path,
            library_cover,
            // Reader
            reader_get_book,
            reader_open_book,
            reader_get_chapter,
            reader_chapter_image,
            reader_chapter_image_path,
            reader_go_to_chapter,
            reader_save_progress,
            reader_get_progress,
            reader_resolve_href,
            // Search / Bookmarks
            search_in_book,
            bookmark_list,
            bookmark_list_all,
            bookmark_add,
            bookmark_remove,
            // Settings
            settings_load,
            settings_save,
            tts_config_load,
            tts_config_save,
            // Assets
            asset_read_file,
            // TTS
            tts_test_connection,
            tts_start,
            tts_pause,
            tts_resume,
            tts_stop,
            tts_clear_cache,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}
