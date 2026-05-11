/*!
EPUB 阅读器主入口
*/

use std::io::Write;
use std::sync::Mutex;

use eframe;
use eframe::egui;
use log::info;

mod app;
mod domain;
mod parser;
mod storage;
mod tts;
mod ui;

/// Writes to two `Write` targets simultaneously.
struct TeeWriter<A: Write, B: Write> {
    a: Mutex<A>,
    b: Mutex<B>,
}

impl<A: Write, B: Write> Write for TeeWriter<A, B> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = self.a.lock().unwrap().write_all(buf);
        let _ = self.b.lock().unwrap().write_all(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.a.lock().unwrap().flush()?;
        self.b.lock().unwrap().flush()?;
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
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("无法创建日志文件");

    let tee = TeeWriter {
        a: Mutex::new(std::io::stderr()),
        b: Mutex::new(file),
    };

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&level_var),
    )
    .target(env_logger::Target::Pipe(Box::new(tee)))
    .init();

    info!("日志启动 (级别={}, 文件={})", level_var, log_path.display());
}

fn main() {
    init_logging();
    info!("X阅读器 启动");

    let _ = storage::paths::ensure_dirs();
    let saved_settings = storage::settings_store::load();

    let mut viewport = egui::ViewportBuilder::default();
    if let Some((w, h)) = saved_settings.window_size {
        viewport = viewport.with_inner_size(egui::vec2(w, h));
    }
    if let Some((x, y)) = saved_settings.window_pos {
        viewport = viewport.with_position(egui::pos2(x, y));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "X阅读器",
        options,
        Box::new(|cc| Ok(Box::new(app::ReaderApp::new(cc)))),
    )
    .unwrap();
}
