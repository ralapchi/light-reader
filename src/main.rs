/*!
EPUB 阅读器主入口

这是一个使用 Rust 和 eframe 框架开发的桌面 EPUB 阅读器，支持打开和显示 EPUB 和 TXT 格式的书籍。
*/

use eframe;
use eframe::egui;
use log::info;

mod app;
mod domain;
mod parser;
mod storage;
mod tts;
mod ui;

/// 应用程序主入口函数
/// 
/// 初始化日志系统并启动 eframe 应用
fn main() {
    env_logger::init();
    info!("EPUB 阅读器启动");

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
        "EPUB 阅读器",
        options,
        Box::new(|cc| Ok(Box::new(app::ReaderApp::new(cc)))),
    )
    .unwrap();
}
