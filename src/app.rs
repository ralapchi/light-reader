use eframe::egui;
use rfd::FileDialog;
use log::{info, debug};

use crate::parser::ParserFactory;

pub struct ReaderApp {
    content: Vec<String>,
    chapter_titles: Vec<String>,
    current_page: usize,
    status: String,
}

impl Default for ReaderApp {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            chapter_titles: Vec::new(),
            current_page: 0,
            status: "就绪".to_string(),
        }
    }
}

impl ReaderApp {
    fn open_book(&mut self, path: &str) {
        self.status = format!("正在打开文件: {}", path);
        info!("{}", self.status);

        match ParserFactory::get_parser(path) {
            Some(parser) => {
                match parser.parse(path) {
                    Ok(result) => {
                        self.current_page = 0;
                        self.content = result.content;
                        self.chapter_titles = result.chapter_titles;
                        
                        info!("找到 {} 个章节", self.chapter_titles.len());
                        self.status = format!("内容已加载，共 {} 章", self.chapter_titles.len());
                        info!("{}", self.status);
                    }
                    Err(e) => {
                        self.status = format!("解析失败: {}", e);
                        info!("{}", self.status);
                    }
                }
            }
            None => {
                self.status = "不支持的文件格式".to_string();
                info!("{}", self.status);
            }
        }
    }

    fn next_page(&mut self) {
        if self.current_page < self.content.len() - 1 {
            self.current_page += 1;
        }
    }

    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    fn configure_fonts(ctx: &egui::Context) {
        debug!("配置中文字体...");
        let font_path = "/System/Library/Fonts/Hiragino Sans GB.ttc";
        debug!("找到字体文件: {}", font_path);
        
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "chinese".to_owned(),
            egui::FontData::from_static(include_bytes!("/System/Library/Fonts/Hiragino Sans GB.ttc")).into(),
        );
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "chinese".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("chinese".to_owned());
        
        ctx.set_fonts(fonts);
        debug!("成功加载中文字体");
        debug!("字体配置完成");
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::configure_fonts(ctx);

        egui::SidePanel::left("toc").default_width(300.0).show(ctx, |ui| {
            ui.heading("目录");
            ui.separator();

            if self.chapter_titles.is_empty() {
                ui.label("请打开 EPUB 文件");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, title) in self.chapter_titles.iter().enumerate() {
                        if ui.selectable_label(idx == self.current_page, title).clicked() {
                            self.current_page = idx;
                        }
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("打开书籍").clicked() {
                    info!("点击了打开书籍按钮");
                    if let Some(path) = FileDialog::new()
                        .add_filter("电子书", &["epub", "txt"])
                        .add_filter("EPUB", &["epub"])
                        .add_filter("文本文件", &["txt"])
                        .pick_file() {
                        let path_str = path.to_str().unwrap_or("");
                        info!("已选择文件: {:?}", path);
                        info!("文件路径: {}", path_str);
                        self.open_book(path_str);
                    }
                }
                ui.separator();
                if ui.button("上一章").clicked() {
                    self.prev_page();
                }
                ui.separator();
                if ui.button("下一章").clicked() {
                    self.next_page();
                }
                ui.separator();
                ui.label(&self.status);
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(content) = self.content.get(self.current_page) {
                    ui.label(content);
                } else {
                    ui.label("请打开 EPUB 文件");
                }
            });
        });
    }
}