/*!
EPUB 阅读器主入口

这是一个使用 Rust 和 eframe 框架开发的桌面 EPUB 阅读器，支持打开和显示 EPUB 和 TXT 格式的书籍。
*/

use eframe;
use log::info;

mod app;
mod domain;
mod parser;
mod storage;
mod ui;

/// 应用程序主入口函数
/// 
/// 初始化日志系统并启动 eframe 应用
fn main() {
    env_logger::init();
    info!("EPUB 阅读器启动");
    
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "EPUB 阅读器",
        options,
        Box::new(|cc| Ok(Box::new(app::ReaderApp::new(cc)))),
    )
    .unwrap();
}
