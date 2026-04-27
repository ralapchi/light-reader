use eframe;
use log::info;

mod app;
mod parser;

fn main() {
    env_logger::init();
    info!("小说 阅读器启动");
    
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "小说 阅读器",
        options,
        Box::new(|_cc| Ok(Box::new(app::ReaderApp::default()))),
    )
    .unwrap();
}